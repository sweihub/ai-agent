//! REPL Bridge - Main entry point for Remote Control functionality.
//!
//! Translated from openclaudecode/src/bridge/replBridge.ts
//!
//! This module provides the main bridge interface for the REPL.

use crate::bridge::SDKMessage;
use crate::bridge::poll_config_defaults::PollIntervalConfig;
use crate::bridge::repl_bridge_handle::{BridgeControlRequest, BridgeControlResponse, BridgeState};
use crate::bridge::repl_bridge_transport::ReplBridgeTransport;
use crate::error::AgentError;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Boxed future type for async operations.
type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output = T> + Send>>;

// =============================================================================
// TYPES
// =============================================================================

/// Options for initializing the REPL bridge.
#[derive(Clone)]
pub struct ReplBridgeOptions {
    /// Current working directory.
    pub dir: String,
    /// Machine name.
    pub machine_name: String,
    /// Current git branch.
    pub branch: String,
    /// Git repo URL.
    pub git_repo_url: Option<String>,
    /// Session title.
    pub title: String,
    /// Base API URL.
    pub base_url: String,
    /// Session ingress URL.
    pub session_ingress_url: String,
    /// Worker type (e.g., "repl", "daemon").
    pub worker_type: String,
    /// Get the current OAuth access token.
    pub get_access_token: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    /// Create a new session.
    pub create_session: Arc<
        dyn Fn(
                String,
                String,
                Option<String>,
                String,
            ) -> BoxFuture<Result<Option<String>, AgentError>>
            + Send
            + Sync,
    >,
    /// Archive a session.
    pub archive_session: Arc<dyn Fn(String) -> BoxFuture<Result<(), AgentError>> + Send + Sync>,
    /// Get current session title (for reconnection).
    pub get_current_title: Option<Arc<dyn Fn() -> String + Send + Sync>>,
    /// Convert internal messages to SDK format.
    pub to_sdk_messages:
        Option<Arc<dyn Fn(Vec<crate::types::Message>) -> Vec<SDKMessage> + Send + Sync>>,
    /// Handle OAuth 401 refresh.
    pub on_auth_401:
        Option<Arc<dyn Fn(String) -> BoxFuture<Result<bool, AgentError>> + Send + Sync>>,
    /// Get poll interval config.
    pub get_poll_interval_config: Option<Arc<dyn Fn() -> PollIntervalConfig + Send + Sync>>,
    /// Max initial messages to replay on connect.
    pub initial_history_cap: Option<u32>,
    /// Initial messages to flush on connect.
    pub initial_messages: Option<Vec<crate::types::Message>>,
    /// Previously flushed UUIDs (for dedup).
    pub previously_flushed_uuids:
        Option<Arc<dyn Fn() -> std::collections::HashSet<String> + Send + Sync>>,
    /// Callback for inbound messages.
    pub on_inbound_message: Option<Arc<dyn Fn(SDKMessage) + Send + Sync>>,
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
    /// Callback for user messages (title derivation).
    pub on_user_message: Option<Arc<dyn Fn(String, String) -> bool + Send + Sync>>,
    /// Whether this is a perpetual (persistent) bridge.
    pub perpetual: Option<bool>,
    /// Initial SSE sequence number (for daemon persistence).
    pub initial_sse_sequence_num: Option<u64>,
}

impl Default for ReplBridgeOptions {
    fn default() -> Self {
        Self {
            dir: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            machine_name: "unknown".to_string(),
            branch: String::new(),
            git_repo_url: None,
            title: String::new(),
            base_url: String::new(),
            session_ingress_url: String::new(),
            worker_type: "repl".to_string(),
            get_access_token: Arc::new(|| None),
            create_session: Arc::new(|_, _, _, _| Box::pin(async { Ok(None) })),
            archive_session: Arc::new(|_| Box::pin(async { Ok(()) })),
            get_current_title: None,
            to_sdk_messages: None,
            on_auth_401: None,
            get_poll_interval_config: None,
            initial_history_cap: None,
            initial_messages: None,
            previously_flushed_uuids: None,
            on_inbound_message: None,
            on_permission_response: None,
            on_interrupt: None,
            on_set_model: None,
            on_set_max_thinking_tokens: None,
            on_set_permission_mode: None,
            on_state_change: None,
            on_user_message: None,
            perpetual: None,
            initial_sse_sequence_num: None,
        }
    }
}

/// Main handle for interacting with the REPL bridge.
pub struct ReplBridge {
    /// The bridge session ID.
    pub session_id: RwLock<String>,
    /// The environment ID.
    pub environment_id: RwLock<String>,
    /// The session ingress URL.
    pub session_ingress_url: String,
    /// Internal handle (if using env-less v2).
    pub inner: RwLock<Option<Arc<dyn ReplBridgeHandleInner>>>,
}

/// Internal bridge handle trait.
pub trait ReplBridgeHandleInner: Send + Sync {
    /// Write messages to the bridge.
    fn write_messages(&self, messages: Vec<SDKMessage>);
    /// Write SDK messages directly.
    fn write_sdk_messages(&self, messages: Vec<SDKMessage>);
    /// Send a control request.
    fn send_control_request(&self, request: BridgeControlRequest);
    /// Send a control response.
    fn send_control_response(&self, response: BridgeControlResponse);
    /// Send a control cancel request.
    fn send_control_cancel_request(&self, request_id: &str);
    /// Send a result message.
    fn send_result(&self);
    /// Tear down the bridge.
    fn teardown(&self) -> BoxFuture<()>;
}

impl ReplBridge {
    /// Create a new REPL bridge.
    pub fn new(session_id: String, environment_id: String, session_ingress_url: String) -> Self {
        Self {
            session_id: RwLock::new(session_id),
            environment_id: RwLock::new(environment_id),
            session_ingress_url,
            inner: RwLock::new(None),
        }
    }

    /// Get the bridge session ID.
    pub async fn bridge_session_id(&self) -> String {
        self.session_id.read().await.clone()
    }

    /// Get the environment ID.
    pub async fn environment_id(&self) -> String {
        self.environment_id.read().await.clone()
    }

    /// Get the session ingress URL.
    pub fn session_ingress_url(&self) -> &str {
        &self.session_ingress_url
    }

    /// Write messages to the bridge.
    pub fn write_messages(&self, messages: Vec<SDKMessage>) {
        if let Some(inner) = self.inner.blocking_read().as_ref() {
            inner.write_messages(messages);
        }
    }

    /// Write SDK messages directly to the bridge.
    pub fn write_sdk_messages(&self, messages: Vec<SDKMessage>) {
        if let Some(inner) = self.inner.blocking_read().as_ref() {
            inner.write_sdk_messages(messages);
        }
    }

    /// Send a control request.
    pub fn send_control_request(&self, request: BridgeControlRequest) {
        if let Some(inner) = self.inner.blocking_read().as_ref() {
            inner.send_control_request(request);
        }
    }

    /// Send a control response.
    pub fn send_control_response(&self, response: BridgeControlResponse) {
        if let Some(inner) = self.inner.blocking_read().as_ref() {
            inner.send_control_response(response);
        }
    }

    /// Send a control cancel request.
    pub fn send_control_cancel_request(&self, request_id: &str) {
        if let Some(inner) = self.inner.blocking_read().as_ref() {
            inner.send_control_cancel_request(request_id);
        }
    }

    /// Send a result message.
    pub fn send_result(&self) {
        if let Some(inner) = self.inner.blocking_read().as_ref() {
            inner.send_result();
        }
    }

    /// Tear down the bridge.
    pub async fn teardown(&self) {
        if let Some(inner) = self.inner.write().await.take() {
            inner.teardown().await;
        }
    }
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Initialize the REPL bridge with the given options.
///
/// Returns None if initialization fails.
pub async fn init_repl_bridge(
    _options: ReplBridgeOptions,
) -> Result<Option<ReplBridge>, AgentError> {
    // Note: The actual implementation would:
    // 1. Check if v2 (env-less) bridge is enabled via GrowthBook
    // 2. If v2: call init_env_less_bridge_core
    // 3. If v1: call init_bridge_core (env-based)
    // 4. Return the appropriate handle

    // For SDK, we return None as actual bridge initialization
    // requires CLI-specific infrastructure
    Ok(None)
}

// =============================================================================
// BRIDGE STATE MANAGEMENT
// =============================================================================

/// Global pointer to the active REPL bridge handle, so callers outside
/// useReplBridge's React tree (tools, slash commands) can invoke handle methods.
static REPL_BRIDGE_HANDLE: std::sync::OnceLock<ReplBridge> = std::sync::OnceLock::new();

/// Set the global REPL bridge handle.
pub fn set_repl_bridge_handle(bridge: Option<ReplBridge>) {
    let _ = REPL_BRIDGE_HANDLE.set(
        bridge.unwrap_or_else(|| ReplBridge::new(String::new(), String::new(), String::new())),
    );
}

/// Get the global REPL bridge handle.
pub fn get_repl_bridge_handle() -> Option<&'static ReplBridge> {
    REPL_BRIDGE_HANDLE.get()
}
