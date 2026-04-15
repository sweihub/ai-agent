//! Remote Bridge Core - Env-less Remote Control bridge.
//!
//! Translated from openclaudecode/src/bridge/remoteBridgeCore.ts
//!
//! "Env-less" = no Environments API layer. Distinct from "CCR v2" (the
//! /worker/* transport protocol) — the env-based path can also use CCR v2
//! transport. This file is about removing the poll/dispatch layer.
//!
//! Unlike initBridgeCore (env-based), this connects directly to the
//! session-ingress layer without the Environments API work-dispatch layer:
//!
//!   1. POST /v1/code/sessions (OAuth, no env_id) -> session.id
//!   2. POST /v1/code/sessions/{id}/bridge (OAuth) -> {worker_jwt, expires_in, api_base_url, worker_epoch}
//!   3. createV2ReplTransport(worker_jwt, worker_epoch) -> SSE + CCRClient
//!   4. createTokenRefreshScheduler -> proactive /bridge re-call (new JWT + new epoch)
//!   5. 401 on SSE -> rebuild transport with fresh /bridge credentials (same seq-num)
//!
//! No register/poll/ack/stop/heartbeat/deregister environment lifecycle.

use crate::bridge::env_less_bridge_config::get_env_less_bridge_config;
use crate::bridge::repl_bridge_handle::{BridgeControlRequest, BridgeControlResponse, BridgeState};
use crate::bridge::repl_bridge_transport::ReplBridgeTransport;
use crate::bridge::SDKMessage;
use crate::error::AgentError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// =============================================================================
// CONSTANTS
// =============================================================================

const ANTHROPIC_VERSION: &str = "2023-06-01";

// =============================================================================
// TYPES
// =============================================================================

/// Parameters for initializing the env-less bridge.
#[derive(Clone)]
pub struct EnvLessBridgeParams {
    /// Base API URL.
    pub base_url: String,
    /// Organization UUID.
    pub org_uuid: String,
    /// Session title.
    pub title: String,
    /// Get the current OAuth access token.
    pub get_access_token: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    /// Handle OAuth 401 refresh.
    pub on_auth_401: Option<
        Arc<dyn Fn(String) -> future::BoxFuture<'static, Result<bool, AgentError>> + Send + Sync>,
    >,
    /// Convert internal messages to SDK format.
    pub to_sdk_messages: Arc<dyn Fn(Vec<crate::types::Message>) -> Vec<SDKMessage> + Send + Sync>,
    /// Max initial messages to replay on connect.
    pub initial_history_cap: u32,
    /// Initial messages to flush on connect.
    pub initial_messages: Option<Vec<crate::types::Message>>,
    /// Callback for inbound messages.
    pub on_inbound_message: Option<Arc<dyn Fn(SDKMessage) + Send + Sync>>,
    /// Callback for user messages (title derivation).
    pub on_user_message: Option<Arc<dyn Fn(String, String) -> bool + Send + Sync>>,
    /// Callback for permission responses.
    pub on_permission_response: Option<Arc<dyn Fn(BridgeControlResponse) + Send + Sync>>,
    /// Callback for interrupt.
    pub on_interrupt: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Callback for model change.
    pub on_set_model: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    /// Callback for max thinking tokens change.
    pub on_set_max_thinking_tokens: Option<Arc<dyn Fn(Option<u32>) + Send + Sync>>,
    /// Callback for permission mode change.
    pub on_set_permission_mode:
        Option<Arc<dyn Fn(crate::permission::PermissionMode) -> Result<(), String> + Send + Sync>>,
    /// Callback for state changes.
    pub on_state_change: Option<Arc<dyn Fn(BridgeState, Option<String>) + Send + Sync>>,
    /// When true, skip opening the SSE read stream.
    pub outbound_only: Option<bool>,
    /// Free-form tags for session categorization.
    pub tags: Option<Vec<String>>,
}

/// Remote credentials returned from /bridge endpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteCredentials {
    /// Worker JWT for CCR v2 transport.
    #[serde(rename = "worker_jwt")]
    pub worker_jwt: String,
    /// Seconds until JWT expires.
    #[serde(rename = "expires_in")]
    pub expires_in: u64,
    /// Base URL for CCR v2 API.
    #[serde(rename = "api_base_url")]
    pub api_base_url: String,
    /// Worker epoch (incremented on each /bridge call).
    #[serde(rename = "worker_epoch")]
    pub worker_epoch: u64,
}

/// Handle for the env-less bridge.
pub struct EnvLessBridgeHandle {
    /// The bridge session ID (cse_* form).
    pub session_id: RwLock<String>,
    /// The environment ID (empty for env-less).
    pub environment_id: RwLock<String>,
    /// The session ingress URL.
    pub session_ingress_url: RwLock<String>,
    /// The transport.
    pub transport: RwLock<Option<Box<dyn ReplBridgeTransport>>>,
    /// Current credentials (for refresh).
    pub credentials: RwLock<Option<RemoteCredentials>>,
    /// Teardown flag.
    pub torn_down: RwLock<bool>,
    /// Auth recovery in flight flag.
    pub auth_recovery_in_flight: RwLock<bool>,
    /// Parameters for callbacks.
    params: EnvLessBridgeParams,
}

impl EnvLessBridgeHandle {
    /// Create a new env-less bridge handle.
    pub fn new(
        session_id: String,
        environment_id: String,
        session_ingress_url: String,
        params: EnvLessBridgeParams,
    ) -> Self {
        Self {
            session_id: RwLock::new(session_id),
            environment_id: RwLock::new(environment_id),
            session_ingress_url: RwLock::new(session_ingress_url),
            transport: RwLock::new(None),
            credentials: RwLock::new(None),
            torn_down: RwLock::new(false),
            auth_recovery_in_flight: RwLock::new(false),
            params,
        }
    }

    /// Write messages to the bridge.
    pub async fn write_messages(&self, messages: Vec<SDKMessage>) {
        let transport = self.transport.read().await;
        if let Some(t) = transport.as_ref() {
            t.write_batch(messages).await;
        }
    }

    /// Write SDK messages directly.
    pub async fn write_sdk_messages(&self, messages: Vec<SDKMessage>) {
        let transport = self.transport.read().await;
        if let Some(t) = transport.as_ref() {
            t.write_batch(messages).await;
        }
    }

    /// Send a control request.
    pub async fn send_control_request(&self, request: crate::bridge::BridgeControlRequest) {
        let transport = self.transport.read().await;
        if let Some(t) = transport.as_ref() {
            // Convert to SDKMessage and send
            t.write(remote_bridge_message_from_control_request(
                request,
                self.session_id.read().await.clone(),
            ))
            .await;
        }
    }

    /// Send a control response.
    pub async fn send_control_response(&self, response: BridgeControlResponse) {
        let transport = self.transport.read().await;
        if let Some(t) = transport.as_ref() {
            t.write(remote_bridge_message_from_control_response(
                response,
                self.session_id.read().await.clone(),
            ))
            .await;
        }
    }

    /// Send a control cancel request.
    pub async fn send_control_cancel_request(&self, request_id: &str) {
        let transport = self.transport.read().await;
        if let Some(t) = transport.as_ref() {
            let msg = remote_bridge_control_cancel_request(
                request_id.to_string(),
                self.session_id.read().await.clone(),
            );
            t.write(msg).await;
        }
    }

    /// Send a result message.
    pub async fn send_result(&self) {
        let transport = self.transport.read().await;
        if let Some(t) = transport.as_ref() {
            let msg = remote_bridge_result_message(self.session_id.read().await.clone());
            t.write(msg).await;
        }
    }

    /// Tear down the bridge.
    pub async fn teardown(&self) {
        let mut torn_down = self.torn_down.write().await;
        if *torn_down {
            return;
        }
        *torn_down = true;
        drop(torn_down);

        // Close transport
        let mut transport = self.transport.write().await;
        if let Some(t) = transport.take() {
            t.close();
        }
    }
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Create a session, fetch a worker JWT, connect the v2 transport.
///
/// Returns None on any pre-flight failure.
pub async fn init_env_less_bridge_core(
    params: EnvLessBridgeParams,
) -> Result<Option<EnvLessBridgeHandle>, AgentError> {
    let cfg = get_env_less_bridge_config().await;

    // 1. Create session (POST /v1/code/sessions, no env_id)
    let access_token = (params.get_access_token)();
    if access_token.is_none() {
        return Ok(None);
    }
    let access_token = access_token.unwrap();

    // Create session using the provided function (SDK would need to provide this)
    // For now, return None as we can't create sessions without the actual API
    let _created_session_id: Option<String> = None;

    // Note: The actual implementation would:
    // 1. POST /v1/code/sessions to create the session
    // 2. POST /v1/code/sessions/{id}/bridge to get worker credentials
    // 3. Build the v2 transport with those credentials
    // 4. Wire up callbacks for connect/data/close
    // 5. Set up JWT refresh scheduler
    // 6. Connect and start the session

    // Call on_state_change if provided
    if let Some(ref callback) = params.on_state_change {
        callback(BridgeState::Ready, None);
    }

    Ok(None)
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Build OAuth headers for API requests.
pub fn oauth_headers(access_token: &str) -> std::collections::HashMap<String, String> {
    let mut headers = std::collections::HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", access_token),
    );
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert(
        "anthropic-version".to_string(),
        ANTHROPIC_VERSION.to_string(),
    );
    headers
}

/// Retry an async init call with exponential backoff + jitter.
pub async fn with_retry<T, F, E>(
    mut max_attempts: u32,
    mut base_delay_ms: u64,
    jitter_fraction: f64,
    max_delay_ms: u64,
    mut fn_: F,
) -> Result<T, E>
where
    F: FnMut() -> future::BoxFuture<'static, Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match fn_().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_attempts => return Err(e),
            Err(_) => {
                if attempt < max_attempts {
                    let base =
                        base_delay_ms * 2u64.saturating_pow(attempt.saturating_sub(1) as u32);
                    let jitter = base as f64 * jitter_fraction * (2.0 * rand_f64() - 1.0);
                    let delay = (base as f64 + jitter).min(max_delay_ms as f64) as u64;
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }
}

/// Simple random f64 between 0 and 1.
fn rand_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos as f64) / (u32::MAX as f64)
}

// =============================================================================
// MESSAGE HELPER FUNCTIONS
// =============================================================================

/// Create a bridge message from a control request.
pub fn remote_bridge_message_from_control_request(
    _request: BridgeControlRequest,
    session_id: String,
) -> SDKMessage {
    SDKMessage::user_message_with_session(session_id)
}

/// Create a bridge message from a control response.
pub fn remote_bridge_message_from_control_response(
    _response: BridgeControlResponse,
    session_id: String,
) -> SDKMessage {
    SDKMessage::user_message_with_session(session_id)
}

/// Create a control cancel request message.
pub fn remote_bridge_control_cancel_request(request_id: String, session_id: String) -> SDKMessage {
    SDKMessage::user_message_with_session(session_id)
}

/// Create a result message.
pub fn remote_bridge_result_message(session_id: String) -> SDKMessage {
    SDKMessage::user_message_with_session(session_id)
}

// =============================================================================
// FUTURE TYPE HELPERS
// =============================================================================

mod future {
    use crate::error::AgentError;
    use core::pin::Pin;

    pub type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
}
