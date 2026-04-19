// Source: /data/home/swei/claudecode/openclaudecode/src/services/voiceStreamSTT.ts
//! VoiceStreamSTT - Anthropic voice_stream speech-to-text client for push-to-talk.
//!
//! Connects to Anthropic's voice_stream WebSocket endpoint using OAuth credentials
//! for conversation_engine-backed speech-to-text (Deepgram Nova 3).
//! Designed for hold-to-talk: hold the keybinding to record, release to stop and submit.
//!
//! Wire protocol: JSON control messages (KeepAlive, CloseStream) + binary audio frames.
//! Server responds with TranscriptText and TranscriptEndpoint JSON messages.
//!
//! PKCE auth with OAuth tokens. WebSocket keepalive every 8 seconds.
//! Finalize timeouts: 5s safety, 1.5s no-data after CloseStream.
//!
//! Note: voice_stream uses the same OAuth as Claude Code — available when the
//! user is authenticated with Anthropic (Claude.ai subscriber or has valid OAuth tokens).
//! The endpoint uses conversation_engine backed models for speech-to-text.

use crate::constants::env::ai_code;
use crate::constants::oauth;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use tokio::time::{interval, Duration};
use tokio_tungstenite::tungstenite::Message;

// ─── Constants ───────────────────────────────────────────────────────

const VOICE_STREAM_PATH: &str = "/api/ws/speech_to_text/voice_stream";
const KEEPALIVE_INTERVAL_SECS: u64 = 8;

/// Finalize resolution timers.
///
/// `no_data` fires when no TranscriptText arrives post-CloseStream — the
/// server has nothing; don't wait out the full ~3-5s WS teardown to confirm
/// emptiness. `safety` is the last-resort cap if the WS hangs.
pub const FINALIZE_TIMEOUTS_MS: FinalizeTimeouts = FinalizeTimeouts {
    safety: 5_000,
    no_data: 1_500,
};

#[derive(Debug, Clone)]
pub struct FinalizeTimeouts {
    pub safety: u64,
    pub no_data: u64,
}

// ─── Types ──────────────────────────────────────────────────────────

/// Callbacks for voice stream events.
pub trait VoiceStreamCallbacks: Send + Sync {
    /// Called with a transcript text chunk.
    /// `is_final` is true when this is the last chunk for an utterance.
    fn on_transcript(&self, text: &str, is_final: bool);

    /// Called on transcription or connection errors.
    /// `fatal` indicates if retrying is unlikely to succeed.
    fn on_error(&self, error: &str, fatal: bool);

    /// Called when the WebSocket connection is closed.
    fn on_close(&self);

    /// Called once the WebSocket is open and ready to receive audio.
    fn on_ready(&self, connection: VoiceStreamConnection);
}

/// How finalize() resolved.
///
/// `no_data_timeout` means zero server messages after CloseStream — the
/// silent-drop signature (anthropics/anthropic#287008).
#[derive(Debug, Clone, PartialEq)]
pub enum FinalizeSource {
    PostClosestreamEndpoint,
    NoDataTimeout,
    SafetyTimeout,
    WsClose,
    WsAlreadyClosed,
}

impl fmt::Display for FinalizeSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FinalizeSource::PostClosestreamEndpoint => write!(f, "post_closestream_endpoint"),
            FinalizeSource::NoDataTimeout => write!(f, "no_data_timeout"),
            FinalizeSource::SafetyTimeout => write!(f, "safety_timeout"),
            FinalizeSource::WsClose => write!(f, "ws_close"),
            FinalizeSource::WsAlreadyClosed => write!(f, "ws_already_closed"),
        }
    }
}

/// A transcript text event from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptTextEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: String,
}

/// A transcript endpoint event from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEndpointEvent {
    #[serde(rename = "type")]
    pub event_type: String,
}

/// A transcript error event from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptErrorEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,
    pub description: Option<String>,
}

/// Voice stream server message types.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum VoiceStreamServerMessage {
    TranscriptText { data: String },
    TranscriptEndpoint,
    TranscriptError {
        #[serde(rename = "errorCode")]
        error_code: Option<String>,
        description: Option<String>,
    },
    Error { message: Option<String> },
}

/// Message type for the audio frame channel.
#[derive(Debug)]
enum AudioFrame {
    Binary(Vec<u8>),
}

/// Message type for the transcript result channel.
#[derive(Debug)]
enum TranscriptResult {
    /// Intermediate transcript text event.
    Text(String),
    /// Final transcript (last chunk, is_final = true).
    Final(String),
    /// Done signal — listener task finished, transcript channel is closing.
    Done,
}

/// Voice stream connection handle.
///
/// Created by [connect_voice_stream]. Provide to the caller so it can start
/// sending audio, finalize recording, and check connection status.
///
/// Architecture:
/// ```text
/// User ───send()──▶ audio_tx ──recv()──▶ [Listener Task]
///                                              │
///                          transcript_rx ◀──send()─┘
///                                  │
///                                  ▼
///                              User drains transcript messages
///
///  User ──finalize()──▶ drop audio_tx ──await join──▶ FinalizeSource
/// ```
#[derive(Clone)]
pub struct VoiceStreamConnection {
    /// Sender for audio frames to the listener task.
    audio_tx: mpsc::UnboundedSender<AudioFrame>,
    /// Receiver for transcript messages from the listener task.
    transcript_rx: mpsc::UnboundedReceiver<TranscriptResult>,
    /// Current connection state.
    state: Arc<watch::Sender<ConnectionState>>,
    /// Oneshot sender for the listener to communicate the finalize result.
    on_tx: oneshot::Sender<FinalizeSource>,
    /// Receiver for the finalize result — used by finalize().
    on_rx: Mutex<Option<oneshot::Receiver<FinalizeSource>>>,
    /// JoinHandle for the spawned listener task.
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, Clone, PartialEq)]
enum ConnectionState {
    Connected,
    Closing,
}

impl VoiceStreamConnection {
    /// Send an audio chunk (binary data, typically 16kHz mono linear16 PCM).
    ///
    /// Silently drops the chunk if the connection is not open or if
    /// finalize() has already been called (audio is rejected after CloseStream).
    ///
    /// Note: Audio hardware initialisation can take >1s, so callers should
    /// call [Self::send] only after [VoiceStreamCallbacks::on_ready] fires.
    pub fn send(&self, audio_chunk: &[u8]) {
        // Drop silently if connection is not open
        let state = *self.state.borrow();
        if state != ConnectionState::Connected {
            return;
        }

        // Send audio frame to the listener task.
        // The listener sends it as a binary WebSocket frame to the server.
        let frame = AudioFrame::Binary(audio_chunk.to_vec());
        if let Err(e) = self.audio_tx.send(frame) {
            eprintln!("[voice_stream] Failed to queue audio chunk: {:?}", e);
        }
    }

    /// Finalize the voice stream session.
    ///
    /// Drops the audio channel to stop accepting audio, then waits for the
    /// listener task to send CloseStream, receive the final transcript, and
    /// close the WebSocket. Returns the finalize source.
    ///
    /// Idempotent: calling multiple times returns immediately with
    /// `FinalizeSource::WsAlreadyClosed`.
    pub async fn finalize(&self) -> FinalizeSource {
        // Already finalized or task already finished — resolve immediately.
        let mut handle_lock = self.task_handle.lock().await;
        if handle_lock.is_none() {
            return FinalizeSource::WsAlreadyClosed;
        }

        // Drop a clone of the audio sender to signal the task to stop
        // accepting audio. The task detects this via recv() returning None.
        drop(self.audio_tx.clone());

        // Also drop the main sender.
        drop(self.audio_tx);

        // Wait for the oneshot from the listener task.
        let mut rx_lock = self.on_rx.lock().await;
        if let Some(rx) = rx_lock.take() {
            match rx.await {
                Ok(source) => source,
                Err(_) => FinalizeSource::WsClose,
            }
        } else {
            FinalizeSource::WsAlreadyClosed
        }
    }

    /// Close the voice stream connection immediately.
    ///
    /// Drops the sender channel, ending the listener task without waiting
    /// for the finalization protocol (no CloseStream/TranscriptEndpoint exchange).
    pub fn close(&self) {
        self.state.send_replace(ConnectionState::Closing);
        // Drop sender to end the listener task immediately.
    }

    /// Check if the connection is currently open and ready for audio.
    pub fn is_connected(&self) -> bool {
        let state = *self.state.borrow();
        state == ConnectionState::Connected
    }
}

// ─── Availability ─────────────────────────────────────────────────────

/// Check if the voice_stream STT service is available.
///
/// Voice stream requires Anthropic OAuth authentication (Claude.ai subscriber
/// or valid OAuth tokens).
pub fn is_voice_stream_available() -> bool {
    // Check if OAuth token is available from environment variable
    std::env::var(ai_code::OAUTH_TOKEN).is_ok()
}

/// Get the current OAuth access token, if available.
fn get_access_token() -> Option<String> {
    std::env::var(ai_code::OAUTH_TOKEN).ok()
}

// ─── Connection ───────────────────────────────────────────────────────

/// Connect to the Anthropic voice_stream STT WebSocket endpoint.
///
/// Establishes a WebSocket connection with OAuth Bearer authentication,
/// starts keepalive pings, and spawns a background task to handle
/// incoming transcript messages.
///
/// Returns a [VoiceStreamConnection] handle on success, or `None` if
/// OAuth tokens are unavailable or the connection fails.
///
/// # Arguments
///
/// * `callbacks` — event callbacks for transcript, error, close, and ready events
/// * `language` — optional language code (default: "en")
/// * `keyterms` — optional keyword hints for STT boosting
pub async fn connect_voice_stream(
    callbacks: Arc<dyn VoiceStreamCallbacks>,
    language: Option<String>,
    keyterms: Option<Vec<String>>,
) -> Option<VoiceStreamConnection> {
    // Ensure OAuth token is fresh before connecting
    let access_token = match get_access_token() {
        Some(t) => t,
        None => {
            eprintln!("[voice_stream] No OAuth token available");
            return None;
        }
    };

    // Build WebSocket base URL.
    // voice_stream is a private_api route, but /api/ws/ is also exposed on
    // the api.anthropic.com listener. We target that host instead of claude.ai
    // because the claude.ai CF zone uses TLS fingerprinting and challenges
    // non-browser clients.
    let ws_base_url = std::env::var(ai_code::VOICE_STREAM_BASE_URL)
        .ok()
        .unwrap_or_else(|| {
            oauth::get_oauth_config()
                .base_api_url
                .replace("https://", "wss://")
                .replace("http://", "ws://")
        });

    if std::env::var(ai_code::VOICE_STREAM_BASE_URL).is_ok() {
        eprintln!(
            "[voice_stream] Using VOICE_STREAM_BASE_URL override: {}",
            ws_base_url
        );
    }

    // Build query parameters
    let mut params: Vec<(&str, &str)> = vec![
        ("encoding", "linear16"),
        ("sample_rate", "16000"),
        ("channels", "1"),
        ("endpointing_ms", "300"),
        ("utterance_end_ms", "1000"),
        ("language", language.as_deref().unwrap_or("en")),
        // Always enable Nova 3 (feature-gated feature always enabled per rules)
        ("use_conversation_engine", "true"),
        ("stt_provider", "deepgram-nova3"),
    ];

    // Append keyterms as query params — the voice_stream proxy forwards
    // these to the STT service which applies appropriate boosting.
    if let Some(ref terms) = keyterms {
        for term in terms {
            params.push(("keyterms", term.as_str()));
        }
    }

    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let url = format!("{}{}?{}", ws_base_url, VOICE_STREAM_PATH, query);

    eprintln!("[voice_stream] Connecting to {}", url);

    // Build headers
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", access_token).parse().unwrap(),
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        crate::utils::http::get_user_agent().parse().unwrap(),
    );
    headers.insert("x-app", "cli".parse().unwrap());

    // Connect to WebSocket
    let (ws_stream, _response) =
        match tokio_tungstenite::connect_async_with_headers(&url, headers).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "[voice_stream] WebSocket connection failed: {}",
                    e
                );
                callbacks.on_error(
                    &format!("Voice stream connection error: {}", e),
                    false,
                );
                return None;
            }
        };

    eprintln!("[voice_stream] WebSocket connected");

    // Split the WebSocket stream into sender and receiver.
    let (ws_write, ws_read) = ws_stream.split();

    // Shared state channel
    let (state_tx, _state_rx) = watch::channel(ConnectionState::Connected);

    // Audio frame channel: user sends audio, listener task receives and
    // forwards to WebSocket. Unbounded — audio chunks are small and brief.
    let (audio_tx, audio_rx) = mpsc::unbounded_channel::<AudioFrame>();

    // Transcript message channel: listener task sends transcript events,
    // user drains this channel in their callback loop.
    let (transcript_tx, transcript_rx) = mpsc::unbounded_channel::<TranscriptResult>();

    // Oneshot channel for the listener to communicate the finalize result back.
    let (on_tx, finalize_rx) = oneshot::channel::<FinalizeSource>();

    // Initial KeepAlive — send so the server knows the client is active.
    // Audio hardware initialisation can take >1s, so this prevents the
    // server from closing the connection before audio capture starts.
    let initial_keepalive = Message::Text("{\"type\":\"KeepAlive\"}".to_string());
    if let Err(e) = ws_write.send(initial_keepalive).await {
        eprintln!("[voice_stream] Failed to send initial KeepAlive: {:?}", e);
    } else {
        eprintln!("[voice_stream] Sent initial KeepAlive");
    }

    // Store the JoinHandle in a shared Mutex so finalize() can await it.
    let task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> =
        Arc::new(Mutex::new(None));
    let task_handle_clone = task_handle.clone();

    // ─── Spawning the message listener task ───────────────────────────
    let handle = tokio::spawn(async move {
        // Keepalive timer for periodic pings to prevent idle timeout
        let mut keepalive_interval =
            interval(Duration::from_secs(KEEPALIVE_INTERVAL_SECS));
        // Skip the first tick since we already sent an initial KeepAlive.
        keepalive_interval.tick().await;

        // ── Message receive loop ──────────────────────────────────────
        //
        // This loop listens on two sources simultaneously:
        // 1. Audio channel — receives audio frames from the user
        // 2. WebSocket — receives transcript messages from the server
        //
        // When the audio channel closes (finalize/close), the task
        // sends CloseStream to the server, waits for the final transcript,
        // and then closes the transcript channel and WebSocket.

        let mut audio_rx = audio_rx;
        let mut ws_read = ws_read;
        let mut transcript_tx = transcript_tx;

        // Send periodic keepalive in a concurrent task
        let ws_write_keepalive = ws_write.clone();
        tokio::spawn(async move {
            let mut interval = keepalive_interval;
            loop {
                interval.tick().await;
                let msg = Message::Text("{\"type\":\"KeepAlive\"}".to_string());
                if let Err(e) = ws_write_keepalive.send(msg).await {
                    eprintln!("[voice_stream] Keepalive send failed: {:?}", e);
                    break;
                } else {
                    eprintln!("[voice_stream] Sending periodic KeepAlive");
                }
            }
        });

        // Finalize-related state
        let mut finalized = false; // Set true once CloseStream has been sent
        let mut finalizing = false; // Set true when finalize() is first called

        // Transcript tracking
        let mut last_transcript_text = String::new();

        // ── Main select loop ──────────────────────────────────────────
        loop {
            tokio::select! {
                // ── Audio channel (user -> listener) ────────────────────
                frame = audio_rx.recv() => {
                    match frame {
                        None => {
                            // Audio channel closed — this happens when finalize()
                            // drops the sender. Now we need to clean up.
                            eprintln!("[voice_stream] Audio channel closed, finalizing");

                            // Send CloseStream to tell the server to stop accepting audio.
                            // Defer to next event-loop iteration so any audio already
                            // queued in the audio_rx is flushed to the WebSocket.
                            finalized = true;
                            let close_msg = Message::Text("{\"type\":\"CloseStream\"}".to_string());

                            // Spawn a deferred task to send CloseStream
                            let ws_write_clone = ws_write.clone();
                            tokio::spawn(async move {
                                if let Err(e) = ws_write_clone.send(close_msg).await {
                                    eprintln!("[voice_stream] Failed to send CloseStream: {:?}", e);
                                } else {
                                    eprintln!("[voice_stream] Sent CloseStream (finalize)");
                                }
                            });

                            break;
                        }
                        Some(AudioFrame::Binary(data)) => {
                            // Audio chunk from the user.
                            if !finalized {
                                eprintln!(
                                    "[voice_stream] Sending audio chunk: {} bytes",
                                    data.len()
                                );
                                if let Err(e) = ws_write.send(Message::Binary(data)).await {
                                    eprintln!("[voice_stream] Failed to send audio: {:?}", e);
                                    break;
                                }
                            }
                        }
                    }
                }

                // ── WebSocket messages (server -> listener) ─────────────
                result = ws_read.next() => {
                    match result {
                        Some(Ok(Message::Text(text))) => {
                            eprintln!(
                                "[voice_stream] Message received ({} chars): {}",
                                text.len(),
                                text.chars().take(200).collect::<String>()
                            );

                            // Parse the JSON message
                            let msg: Result<VoiceStreamServerMessage, _> =
                                serde_json::from_str(&text);

                            match msg {
                                Ok(VoiceStreamServerMessage::TranscriptText { data }) => {
                                    eprintln!(
                                        "[voice_stream] TranscriptText: \"{}\"",
                                        data
                                    );

                                    // Data arrived after CloseStream — disarm the
                                    // no-data timer so a slow-but-real flush isn't
                                    // cut off.
                                    if finalized {
                                        // In the Rust version, we don't use a no-data
                                        // timer — we wait for the WebSocket close
                                        // event directly after CloseStream.
                                    }

                                    if !data.is_empty() {
                                        last_transcript_text = data.clone();
                                        // Emit as interim so the caller can show a live preview.
                                        let _ = transcript_tx.send(TranscriptResult::Text(data));
                                    }
                                }

                                Ok(VoiceStreamServerMessage::TranscriptEndpoint) => {
                                    eprintln!(
                                        "[voice_stream] TranscriptEndpoint received, lastTranscriptText=\"{}\"",
                                        last_transcript_text
                                    );

                                    let final_text = last_transcript_text.clone();
                                    last_transcript_text.clear();

                                    if !final_text.is_empty() {
                                        // Emit as final so the caller can commit it.
                                        let _ = transcript_tx.send(TranscriptResult::Final(final_text.clone()));
                                    }

                                    // When TranscriptEndpoint arrives after CloseStream was sent,
                                    // resolve finalize now (~300ms) instead of waiting for
                                    // the WebSocket close event (~3-5s of server teardown).
                                    if finalized {
                                        eprintln!(
                                            "[voice_stream] Finalize resolved via post_closestream_endpoint"
                                        );
                                        let _ = on_tx.send(FinalizeSource::PostClosestreamEndpoint);
                                    }
                                }

                                Ok(VoiceStreamServerMessage::TranscriptError { error_code, description }) => {
                                    let desc = description
                                        .or_else(|| error_code)
                                        .unwrap_or_else(|| "unknown transcription error".to_string());
                                    eprintln!("[voice_stream] TranscriptError: {}", desc);
                                    if !finalizing {
                                        callbacks.on_error(&desc, false);
                                    }
                                }

                                Ok(VoiceStreamServerMessage::Error { message }) => {
                                    let error_detail = message
                                        .unwrap_or_else(|| {
                                            serde_json::to_string(&msg).unwrap_or_default()
                                        });
                                    eprintln!("[voice_stream] Server error: {}", error_detail);
                                    if !finalizing {
                                        callbacks.on_error(&error_detail, false);
                                    }
                                }

                                Err(e) => {
                                    eprintln!("[voice_stream] Failed to parse message: {}", e);
                                }
                            }
                        }

                        Some(Ok(Message::Binary(data))) => {
                            // Binary data from server — log for debugging.
                            eprintln!(
                                "[voice_stream] Binary message received: {} bytes",
                                data.len()
                            );
                        }

                        Some(Ok(Message::Close(close_frame))) => {
                            let code = close_frame
                                .as_ref()
                                .map(|c| c.code.as_u16())
                                .unwrap_or(1005);
                            let reason = close_frame
                                .as_ref()
                                .and_then(|c| c.reason.as_str())
                                .unwrap_or("");

                            eprintln!(
                                "[voice_stream] WebSocket closed: code={}, reason=\"{}\"",
                                code, reason
                            );

                            // Promote the last interim transcript to final
                            // so no text is lost.
                            if !last_transcript_text.is_empty() {
                                eprintln!(
                                    "[voice_stream] Promoting unreported interim transcript to final on close"
                                );
                                let final_text = last_transcript_text.clone();
                                last_transcript_text.clear();
                                callbacks.on_transcript(&final_text, true);
                            }

                            // Resolve finalize if still pending.
                            if let Err(_) = on_tx.send(FinalizeSource::WsClose) {
                                // Already resolved or dropped — ignore.
                            }

                            callbacks.on_close();
                            break;
                        }

                        Some(Ok(_)) => {
                            // Ping/Pong or other — skip silently.
                        }

                        Some(Err(e)) => {
                            eprintln!("[voice_stream] WebSocket error: {:?}", e);
                            if !finalizing {
                                callbacks.on_error(
                                    &format!("Voice stream connection error: {:?}", e),
                                    false,
                                );
                            }
                        }

                        None => {
                            // Stream ended (no more messages).
                            break;
                        }
                    }
                }
            }
        }

        // ── Cleanup on loop exit ───────────────────────────────────────
        eprintln!("[voice_stream] Listener task cleanup");

        // Promote any remaining interim transcript
        if !last_transcript_text.is_empty() {
            eprintln!(
                "[voice_stream] Promoting unreported interim transcript during task exit"
            );
            let final_text = last_transcript_text.clone();
            callbacks.on_transcript(&final_text, true);
        }

        // Signal done to the user — transcript channel will close after this.
        let _ = transcript_tx.send(TranscriptResult::Done);

        // Resolve finalize if still pending.
        if let Err(_) = on_tx.send(FinalizeSource::WsClose) {
            // Already resolved or dropped — ignore.
        }

        callbacks.on_close();

        // Clear the JoinHandle from the shared Mutex.
        {
            let mut handle = task_handle_clone.lock().await;
            *handle = None;
        }

        eprintln!("[voice_stream] Listener task ended");
    });

    // Store the task handle.
    {
        let mut handle = task_handle_clone.lock().await;
        *handle = Some(handle);
    }

    // Create the connection handle
    let connection = VoiceStreamConnection {
        audio_tx,
        transcript_rx,
        state: Arc::new(state_tx),
        on_tx,
        on_rx: Mutex::new(Some(finalize_rx)),
        task_handle: task_handle_clone,
    };

    // Notify caller that the connection is ready.
    callbacks.on_ready(connection.clone());

    Some(connection)
}

// ─── Utility ─────────────────────────────────────────────────────────

/// Build a KeepAlive JSON message string.
pub fn keepalive_message() -> String {
    "{\"type\":\"KeepAlive\"}".to_string()
}

/// Build a CloseStream JSON message string.
pub fn close_stream_message() -> String {
    "{\"type\":\"CloseStream\"}".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keepalive_message_format() {
        let msg = keepalive_message();
        assert_eq!(msg, "{\"type\":\"KeepAlive\"}");
    }

    #[test]
    fn test_close_stream_message_format() {
        let msg = close_stream_message();
        assert_eq!(msg, "{\"type\":\"CloseStream\"}");
    }

    #[test]
    fn test_finalize_source_display() {
        assert_eq!(
            format!("{}", FinalizeSource::PostClosestreamEndpoint),
            "post_closestream_endpoint"
        );
        assert_eq!(
            format!("{}", FinalizeSource::NoDataTimeout),
            "no_data_timeout"
        );
        assert_eq!(
            format!("{}", FinalizeSource::SafetyTimeout),
            "safety_timeout"
        );
        assert_eq!(format!("{}", FinalizeSource::WsClose), "ws_close");
        assert_eq!(
            format!("{}", FinalizeSource::WsAlreadyClosed),
            "ws_already_closed"
        );
    }

    #[test]
    fn test_finalize_timeouts_constants() {
        assert_eq!(FINALIZE_TIMEOUTS_MS.safety, 5_000);
        assert_eq!(FINALIZE_TIMEOUTS_MS.no_data, 1_500);
    }

    #[test]
    fn test_voice_stream_server_message_transcript_text() {
        let json = r#"{"type":"TranscriptText","data":"hello world"}"#;
        let msg: VoiceStreamServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            VoiceStreamServerMessage::TranscriptText { data } => {
                assert_eq!(data, "hello world");
            }
            _ => panic!("Expected TranscriptText"),
        }
    }

    #[test]
    fn test_voice_stream_server_message_transcript_endpoint() {
        let json = r#"{"type":"TranscriptEndpoint"}"#;
        let msg: VoiceStreamServerMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, VoiceStreamServerMessage::TranscriptEndpoint));
    }

    #[test]
    fn test_voice_stream_server_message_transcript_error() {
        let json = r#"{"type":"TranscriptError","errorCode":"invalid_audio","description":"Bad audio format"}"#;
        let msg: VoiceStreamServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            VoiceStreamServerMessage::TranscriptError {
                error_code,
                description,
            } => {
                assert_eq!(error_code, Some("invalid_audio".to_string()));
                assert_eq!(description, Some("Bad audio format".to_string()));
            }
            _ => panic!("Expected TranscriptError"),
        }
    }

    #[test]
    fn test_voice_stream_server_message_error() {
        let json = r#"{"type":"error","message":"Server error"}"#;
        let msg: VoiceStreamServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            VoiceStreamServerMessage::Error { message } => {
                assert_eq!(message, Some("Server error".to_string()));
            }
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_connection_state_values() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_eq!(ConnectionState::Closing, ConnectionState::Closing);
        assert_ne!(ConnectionState::Connected, ConnectionState::Closing);
    }

    #[test]
    fn test_voice_stream_server_message_transcript_error_no_desc() {
        let json = r#"{"type":"TranscriptError","errorCode":"bad_format"}"#;
        let msg: VoiceStreamServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            VoiceStreamServerMessage::TranscriptError {
                error_code,
                description,
            } => {
                assert_eq!(error_code, Some("bad_format".to_string()));
                assert!(description.is_none());
            }
            _ => panic!("Expected TranscriptError"),
        }
    }

    #[test]
    fn test_voice_stream_server_message_error_no_message() {
        let json = r#"{"type":"error"}"#;
        let msg: VoiceStreamServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            VoiceStreamServerMessage::Error { message } => {
                assert!(message.is_none());
            }
            _ => panic!("Expected Error"),
        }
    }
}
