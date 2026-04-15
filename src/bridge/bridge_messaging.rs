//! Shared transport-layer helpers for bridge message handling.
//!
//! Translated from openclaudecode/src/bridge/bridgeMain.ts
//!
//! Extracted from replBridge.ts so both the env-based core (initBridgeCore)
//! and the env-less core (initEnvLessBridgeCore) can use the same ingress
//! parsing, control-request handling, and echo-dedup machinery.
//!
//! Everything here is pure — no closure over bridge-specific state. All
//! collaborators (transport, sessionId, UUID sets, callbacks) are passed
//! as params.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

use crate::bridge::SDKMessage;

// =============================================================================
// TYPE GUARDS
// =============================================================================

/// Type predicate for parsed WebSocket messages. SDKMessage is a
/// discriminated union on `type` — validating the discriminant is
/// sufficient for the predicate; callers narrow further via the union.
pub fn is_sdk_message(value: &serde_json::Value) -> bool {
    value.get("type").and_then(|v| v.as_str()).is_some()
}

/// Type predicate for control_response messages from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKControlResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub response: SDKControlResponsePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKControlResponsePayload {
    #[serde(rename = "subtype")]
    pub response_subtype: String,
    #[serde(rename = "request_id")]
    pub request_id: String,
    pub error: Option<String>,
    pub response: Option<serde_json::Value>,
}

impl SDKControlResponse {
    pub fn new(subtype: &str, request_id: &str) -> Self {
        Self {
            response_type: "control_response".to_string(),
            response: SDKControlResponsePayload {
                response_subtype: subtype.to_string(),
                request_id: request_id.to_string(),
                error: None,
                response: None,
            },
        }
    }

    pub fn success(request_id: &str) -> Self {
        Self::new("success", request_id)
    }

    pub fn error(request_id: &str, error: &str) -> Self {
        Self {
            response_type: "control_response".to_string(),
            response: SDKControlResponsePayload {
                response_subtype: "error".to_string(),
                request_id: request_id.to_string(),
                error: Some(error.to_string()),
                response: None,
            },
        }
    }
}

pub fn is_sdk_control_response(value: &serde_json::Value) -> bool {
    value
        .get("type")
        .and_then(|v| v.as_str())
        .map(|s| s == "control_response")
        .unwrap_or(false)
        && value.get("response").is_some()
}

/// Type predicate for control_request messages from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SDKControlRequest {
    ControlRequest {
        request_id: String,
        request: SDKControlRequestPayload,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKControlRequestPayload {
    #[serde(rename = "subtype")]
    pub request_subtype: String,
    pub model: Option<String>,
    #[serde(rename = "max_thinking_tokens")]
    pub max_thinking_tokens: Option<u32>,
    pub mode: Option<String>,
}

pub fn is_sdk_control_request(value: &serde_json::Value) -> bool {
    value
        .get("type")
        .and_then(|v| v.as_str())
        .map(|s| s == "control_request")
        .unwrap_or(false)
        && value.get("request_id").is_some()
        && value.get("request").is_some()
}

// =============================================================================
// MESSAGE ELIGIBILITY
// =============================================================================

/// Message type for internal representation
#[derive(Debug, Clone)]
pub enum MessageType {
    User,
    Assistant,
    System,
    ToolUse,
    ToolResult,
}

/// Check if a message type should be forwarded to the bridge transport.
/// The server only wants user/assistant turns and slash-command system events;
/// everything else (tool_result, progress, etc.) is internal REPL chatter.
pub fn is_eligible_bridge_message(
    msg_type: &MessageType,
    is_virtual: bool,
    system_subtype: Option<&str>,
) -> bool {
    // Virtual messages (REPL inner calls) are display-only — bridge/SDK
    // consumers see the REPL tool_use/result which summarizes the work.
    if matches!(msg_type, MessageType::User | MessageType::Assistant) && is_virtual {
        return false;
    }
    matches!(msg_type, MessageType::User | MessageType::Assistant)
        || (matches!(msg_type, MessageType::System) && system_subtype == Some("local_command"))
}

// =============================================================================
// TITLE TEXT EXTRACTION
// =============================================================================

/// Extract title-worthy text from a Message for onUserMessage. Returns
/// None for messages that shouldn't title the session: non-user, meta
/// (nudges), tool results, compact summaries, non-human origins (task
/// notifications, channel messages), or pure display-tag content
/// (<ide_opened_file>, <session-start-hook>, etc.).
pub fn extract_title_text(
    msg_type: &MessageType,
    is_meta: bool,
    tool_use_result: bool,
    is_compact_summary: bool,
    origin_kind: Option<&str>,
    content: &str,
) -> Option<String> {
    // Filter out non-user, meta, tool results, compact summaries
    if !matches!(msg_type, MessageType::User) || is_meta || tool_use_result || is_compact_summary {
        return None;
    }

    // Filter out non-human origins
    if let Some(kind) = origin_kind {
        if kind != "human" {
            return None;
        }
    }

    // Extract text content
    if content.is_empty() {
        return None;
    }

    // Strip display tags (simplified - would need full implementation)
    let clean = strip_display_tags_allow_empty(content);
    if clean.is_empty() {
        None
    } else {
        Some(clean)
    }
}

/// Strip display tags from text (simplified implementation).
fn strip_display_tags_allow_empty(s: &str) -> String {
    // Simplified: just return the input for now
    // Full implementation would strip <ide_opened_file>, <session-start-hook>, etc.
    s.to_string()
}

// =============================================================================
// INGRESS ROUTING
// =============================================================================

/// Ingress message handler callback types
pub type OnInboundMessage = Arc<dyn Fn(SDKMessage) + Send + Sync>;
pub type OnPermissionResponse = Arc<dyn Fn(SDKControlResponse) + Send + Sync>;
pub type OnControlRequest = Arc<dyn Fn(SDKControlRequest) + Send + Sync>;

/// Parse an ingress WebSocket message and route it to the appropriate handler.
/// Ignores messages whose UUID is in recentPostedUUIDs (echoes of what we sent)
/// or in recentInboundUUIDs (re-deliveries we've already forwarded — e.g.
/// server replayed history after a transport swap lost the seq-num cursor).
pub fn handle_ingress_message(
    data: &str,
    recent_posted_uuids: &mut BoundedUuidSet,
    recent_inbound_uuids: &mut BoundedUuidSet,
    on_inbound_message: Option<&OnInboundMessage>,
    on_permission_response: Option<&OnPermissionResponse>,
    on_control_request: Option<&OnControlRequest>,
    log_for_debugging: &dyn Fn(&str),
) {
    // Parse the JSON data
    let parsed: serde_json::Value = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(err) => {
            log_for_debugging(&format!(
                "[bridge:repl] Failed to parse ingress message: {}",
                err
            ));
            return;
        }
    };

    // control_response is not an SDKMessage — check before the type guard
    if is_sdk_control_response(&parsed) {
        log_for_debugging("[bridge:repl] Ingress message type=control_response");
        if let Some(callback) = on_permission_response {
            if let Ok(response) = serde_json::from_value::<SDKControlResponse>(parsed.clone()) {
                callback(response);
            }
        }
        return;
    }

    // control_request from the server (initialize, set_model, can_use_tool).
    // Must respond promptly or the server kills the WS (~10-14s timeout).
    if is_sdk_control_request(&parsed) {
        let subtype = parsed
            .get("request")
            .and_then(|r| r.get("subtype"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        log_for_debugging(&format!(
            "[bridge:repl] Inbound control_request subtype={}",
            subtype
        ));
        if let Some(callback) = on_control_request {
            if let Ok(request) = serde_json::from_value::<SDKControlRequest>(parsed.clone()) {
                callback(request);
            }
        }
        return;
    }

    if !is_sdk_message(&parsed) {
        return;
    }

    // Check for UUID to detect echoes of our own messages
    let uuid = parsed.get("uuid").and_then(|v| v.as_str());

    if let Some(uuid_str) = uuid {
        if recent_posted_uuids.contains(uuid_str) {
            let msg_type = parsed
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            log_for_debugging(&format!(
                "[bridge:repl] Ignoring echo: type={} uuid={}",
                msg_type, uuid_str
            ));
            return;
        }

        // Defensive dedup: drop inbound prompts we've already forwarded.
        if recent_inbound_uuids.contains(uuid_str) {
            let msg_type = parsed
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            log_for_debugging(&format!(
                "[bridge:repl] Ignoring re-delivered inbound: type={} uuid={}",
                msg_type, uuid_str
            ));
            return;
        }
    }

    let msg_type = parsed
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let uuid_suffix = uuid.map(|u| format!(" uuid={}", u)).unwrap_or_default();
    log_for_debugging(&format!(
        "[bridge:repl] Ingress message type={}{}",
        msg_type, uuid_suffix
    ));

    if msg_type == "user" {
        if let Some(uuid_str) = uuid {
            recent_inbound_uuids.add(uuid_str.to_string());
        }
        // Fire-and-forget — handler may be async (attachment resolution).
        if let Some(callback) = on_inbound_message {
            if let Ok(msg) = serde_json::from_value::<SDKMessage>(parsed.clone()) {
                callback(msg);
            }
        }
    } else {
        log_for_debugging(&format!(
            "[bridge:repl] Ignoring non-user inbound message: type={}",
            msg_type
        ));
    }
}

// =============================================================================
// SERVER-INITIATED CONTROL REQUESTS
// =============================================================================

/// Server control request handlers
pub struct ServerControlRequestHandlers {
    pub transport: Option<Box<dyn ReplBridgeTransport + Send>>,
    pub session_id: String,
    /// When true, all mutable requests (interrupt, set_model, set_permission_mode,
    /// set_max_thinking_tokens) reply with an error instead of false-success.
    /// initialize still replies success — the server kills the connection otherwise.
    /// Used by the outbound-only bridge mode and the SDK's /bridge subpath so claude.ai sees a
    /// proper error instead of "action succeeded but nothing happened locally".
    pub outbound_only: bool,
    pub on_interrupt: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_set_model: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    pub on_set_max_thinking_tokens: Option<Arc<dyn Fn(Option<u32>) + Send + Sync>>,
    pub on_set_permission_mode: Option<Arc<dyn Fn(String) -> Result<(), String> + Send + Sync>>,
}

/// Trait for bridge transport
pub trait ReplBridgeTransport {
    fn write(&self, event: serde_json::Value) -> Result<(), String>;
}

const OUTBOUND_ONLY_ERROR: &str =
    "This session is outbound-only. Enable Remote Control locally to allow inbound control.";

/// Respond to inbound control_request messages from the server. The server
/// sends these for session lifecycle events (initialize, set_model) and
/// for turn-level coordination (interrupt, set_max_thinking_tokens). If we
/// don't respond, the server hangs and kills the WS after ~10-14s.
pub fn handle_server_control_request(
    request: &SDKControlRequest,
    handlers: &ServerControlRequestHandlers,
    log_for_debugging: &dyn Fn(&str),
) {
    let ServerControlRequestHandlers {
        transport,
        session_id,
        outbound_only,
        on_interrupt,
        on_set_model,
        on_set_max_thinking_tokens,
        on_set_permission_mode,
    } = handlers;

    let Some(transport) = transport else {
        log_for_debugging(
            "[bridge:repl] Cannot respond to control_request: transport not configured",
        );
        return;
    };

    let SDKControlRequest::ControlRequest {
        request_id,
        request: request_payload,
    } = request
    else {
        return;
    };

    let request_subtype = &request_payload.request_subtype;

    let response: SDKControlResponse;

    // Outbound-only: reply error for mutable requests so claude.ai doesn't show
    // false success. initialize must still succeed (server kills the connection
    // if it doesn't — see comment above).
    if *outbound_only && request_subtype != "initialize" {
        response = SDKControlResponse {
            response_type: "control_response".to_string(),
            response: SDKControlResponsePayload {
                response_subtype: "error".to_string(),
                request_id: request_id.clone(),
                error: Some(OUTBOUND_ONLY_ERROR.to_string()),
                response: None,
            },
        };
        let event = serde_json::json!({
            "type": "control_response",
            "response": response.response,
            "session_id": session_id
        });
        let _ = transport.write(event);
        log_for_debugging(&format!(
            "[bridge:repl] Rejected {} (outbound-only) request_id={}",
            request_subtype, request_id
        ));
        return;
    }

    match request_subtype.as_str() {
        "initialize" => {
            // Respond with minimal capabilities — the REPL handles
            // commands, models, and account info itself.
            response = SDKControlResponse {
                response_type: "control_response".to_string(),
                response: SDKControlResponsePayload {
                    response_subtype: "success".to_string(),
                    request_id: request_id.clone(),
                    error: None,
                    response: Some(serde_json::json!({
                        "commands": [],
                        "output_style": "normal",
                        "available_output_styles": ["normal"],
                        "models": [],
                        "account": {},
                        "pid": std::process::id(),
                    })),
                },
            };
        }
        "set_model" => {
            on_set_model
                .as_ref()
                .map(|cb| cb(request_payload.model.clone()));
            response = SDKControlResponse {
                response_type: "control_response".to_string(),
                response: SDKControlResponsePayload {
                    response_subtype: "success".to_string(),
                    request_id: request_id.clone(),
                    error: None,
                    response: None,
                },
            };
        }
        "set_max_thinking_tokens" => {
            on_set_max_thinking_tokens
                .as_ref()
                .map(|cb| cb(request_payload.max_thinking_tokens));
            response = SDKControlResponse {
                response_type: "control_response".to_string(),
                response: SDKControlResponsePayload {
                    response_subtype: "success".to_string(),
                    request_id: request_id.clone(),
                    error: None,
                    response: None,
                },
            };
        }
        "set_permission_mode" => {
            // The callback returns a policy verdict so we can send an error
            // control_response without importing isAutoModeGateEnabled /
            // isBypassPermissionsModeDisabled here (bootstrap-isolation). If no
            // callback is registered (daemon context, which doesn't wire this),
            // return an error verdict rather than a silent false-success: the mode
            // is never actually applied in that context, so success would lie to the client.
            let mode = request_payload.mode.clone().unwrap_or_default();
            let verdict = on_set_permission_mode
                .as_ref()
                .map(|cb| cb(mode.clone()))
                .unwrap_or(Err(
                    "set_permission_mode is not supported in this context (onSetPermissionMode callback not registered)".to_string()
                ));

            if verdict.is_ok() {
                response = SDKControlResponse {
                    response_type: "control_response".to_string(),
                    response: SDKControlResponsePayload {
                        response_subtype: "success".to_string(),
                        request_id: request_id.clone(),
                        error: None,
                        response: None,
                    },
                };
            } else {
                response = SDKControlResponse {
                    response_type: "control_response".to_string(),
                    response: SDKControlResponsePayload {
                        response_subtype: "error".to_string(),
                        request_id: request_id.clone(),
                        error: Some(verdict.err().unwrap_or_default()),
                        response: None,
                    },
                };
            }
        }
        "interrupt" => {
            on_interrupt.as_ref().map(|cb| cb());
            response = SDKControlResponse {
                response_type: "control_response".to_string(),
                response: SDKControlResponsePayload {
                    response_subtype: "success".to_string(),
                    request_id: request_id.clone(),
                    error: None,
                    response: None,
                },
            };
        }
        _ => {
            // Unknown subtype — respond with error so the server doesn't
            // hang waiting for a reply that never comes.
            response = SDKControlResponse {
                response_type: "control_response".to_string(),
                response: SDKControlResponsePayload {
                    response_subtype: "error".to_string(),
                    request_id: request_id.clone(),
                    error: Some(format!(
                        "REPL bridge does not handle control_request subtype: {}",
                        request_subtype
                    )),
                    response: None,
                },
            };
        }
    }

    let event = serde_json::json!({
        "type": "control_response",
        "response": response.response,
        "session_id": session_id
    });
    let _ = transport.write(event);
    log_for_debugging(&format!(
        "[bridge:repl] Sent control_response for {} request_id={} result={}",
        request_subtype, request_id, request_payload.request_subtype
    ));
}

// =============================================================================
// RESULT MESSAGE (for session archival on teardown)
// =============================================================================

/// Empty usage for result message
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmptyUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(rename = "cache_creation_input_tokens")]
    pub cache_creation_input_tokens: u32,
    #[serde(rename = "cache_hit_input_tokens")]
    pub cache_hit_input_tokens: u32,
}

/// Build a minimal `SDKResultSuccess` message for session archival.
/// The server needs this event before a WS close to trigger archival.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDKResultSuccess {
    #[serde(rename = "type")]
    pub result_type: String,
    pub subtype: String,
    #[serde(rename = "duration_ms")]
    pub duration_ms: u64,
    #[serde(rename = "duration_api_ms")]
    pub duration_api_ms: u64,
    #[serde(rename = "is_error")]
    pub is_error: bool,
    #[serde(rename = "num_turns")]
    pub num_turns: u32,
    pub result: String,
    #[serde(rename = "stop_reason")]
    pub stop_reason: Option<String>,
    #[serde(rename = "total_cost_usd")]
    pub total_cost_usd: f64,
    pub usage: EmptyUsage,
    #[serde(rename = "model_usage")]
    pub model_usage: serde_json::Value,
    #[serde(rename = "permission_denials")]
    pub permission_denials: Vec<String>,
    #[serde(rename = "session_id")]
    pub session_id: String,
    pub uuid: String,
}

pub fn make_result_message(session_id: &str) -> SDKResultSuccess {
    SDKResultSuccess {
        result_type: "result".to_string(),
        subtype: "success".to_string(),
        duration_ms: 0,
        duration_api_ms: 0,
        is_error: false,
        num_turns: 0,
        result: String::new(),
        stop_reason: None,
        total_cost_usd: 0.0,
        usage: EmptyUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_hit_input_tokens: 0,
        },
        model_usage: serde_json::json!({}),
        permission_denials: vec![],
        session_id: session_id.to_string(),
        uuid: uuid::Uuid::new_v4().to_string(),
    }
}

// =============================================================================
// BOUNDED UUID SET (echo-dedup ring buffer)
// =============================================================================

/// FIFO-bounded set backed by a circular buffer. Evicts the oldest entry
/// when capacity is reached, keeping memory usage constant at O(capacity).
///
/// Messages are added in chronological order, so evicted entries are always
/// the oldest. The caller relies on external ordering (the hook's
/// lastWrittenIndexRef) as the primary dedup — this set is a secondary
/// safety net for echo filtering and race-condition dedup.
pub struct BoundedUuidSet {
    capacity: usize,
    ring: Vec<Option<String>>,
    set: HashSet<String>,
    write_idx: usize,
}

impl BoundedUuidSet {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            ring: vec![None; capacity],
            set: HashSet::new(),
            write_idx: 0,
        }
    }

    pub fn add(&mut self, uuid: String) {
        if self.set.contains(&uuid) {
            return;
        }
        // Evict the entry at the current write position (if occupied)
        if let Some(evicted) = self.ring[self.write_idx].take() {
            self.set.remove(&evicted);
        }
        self.ring[self.write_idx] = Some(uuid.clone());
        self.set.insert(uuid);
        self.write_idx = (self.write_idx + 1) % self.capacity;
    }

    pub fn contains(&self, uuid: &str) -> bool {
        self.set.contains(uuid)
    }

    pub fn clear(&mut self) {
        self.set.clear();
        for item in &mut self.ring {
            *item = None;
        }
        self.write_idx = 0;
    }
}

impl Default for BoundedUuidSet {
    fn default() -> Self {
        Self::new(100)
    }
}
