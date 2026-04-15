//! Bridge types.
//!
//! Translated from openclaudecode/src/bridge/types.ts

use serde::{Deserialize, Serialize};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Default per-session timeout (24 hours).
pub const DEFAULT_SESSION_TIMEOUT_MS: u64 = 24 * 60 * 60 * 1000;

/// Reusable login guidance appended to bridge auth errors.
pub const BRIDGE_LOGIN_INSTRUCTION: &str = "Remote Control is only available with \
    claude.ai subscriptions. Please use `/login` to sign in with your claude.ai account.";

/// Full error printed when `claude remote-control` is run without auth.
pub const BRIDGE_LOGIN_ERROR: &str = "Error: You must be logged in to use Remote Control.\n\n\
    Remote Control is only available with claude.ai subscriptions. Please use `/login` to \
    sign in with your claude.ai account.";

/// Shown when the user disconnects Remote Control (via /remote-control or ultraplan launch).
pub const REMOTE_CONTROL_DISCONNECTED_MSG: &str = "Remote Control disconnected.";

// =============================================================================
// PROTOCOL TYPES FOR THE ENVIRONMENTS API
// =============================================================================

/// Work data from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkData {
    #[serde(rename = "type")]
    pub data_type: String,
    pub id: String,
}

/// Work response from poll endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    #[serde(rename = "environment_id")]
    pub environment_id: String,
    pub state: String,
    pub data: WorkData,
    pub secret: String, // base64url-encoded JSON
    #[serde(rename = "created_at")]
    pub created_at: String,
}

/// Work secret decoded from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSecret {
    pub version: u32,
    #[serde(rename = "session_ingress_token")]
    pub session_ingress_token: String,
    #[serde(rename = "api_base_url")]
    pub api_base_url: String,
    pub sources: Vec<WorkSource>,
    pub auth: Vec<WorkAuth>,
    #[serde(rename = "claude_code_args")]
    pub claude_code_args: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "mcp_config")]
    pub mcp_config: Option<serde_json::Value>,
    #[serde(rename = "environment_variables")]
    pub environment_variables: Option<std::collections::HashMap<String, String>>,
    /// Server-driven CCR v2 selector. Set by prepare_work_secret() when the
    /// session was created via the v2 compat layer (ccr_v2_compat_enabled).
    /// Same field the BYOC runner reads at environment-runner/sessionExecutor.ts.
    #[serde(rename = "use_code_sessions")]
    pub use_code_sessions: Option<bool>,
}

/// Work source (e.g., git repository)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(rename = "git_info")]
    pub git_info: Option<GitInfo>,
}

/// Git info for work source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    #[serde(rename = "type")]
    pub info_type: String,
    pub repo: String,
    pub r#ref: Option<String>,
    pub token: Option<String>,
}

/// Work auth entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub token: String,
}

/// Session done status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionDoneStatus {
    Completed,
    Failed,
    Interrupted,
}

/// Session activity type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionActivityType {
    ToolStart,
    Text,
    Result,
    Error,
}

/// Session activity for displaying tool progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    #[serde(rename = "type")]
    pub activity_type: SessionActivityType,
    /// e.g. "Editing src/foo.ts", "Reading package.json"
    pub summary: String,
    pub timestamp: u64,
}

// =============================================================================
// SPAWN MODE
// =============================================================================

/// How `claude remote-control` chooses session working directories.
/// - `single-session`: one session in cwd, bridge tears down when it ends
/// - `worktree`: persistent server, every session gets an isolated git worktree
/// - `same-dir`: persistent server, every session shares cwd (can stomp each other)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpawnMode {
    SingleSession,
    Worktree,
    SameDir,
}

impl Default for SpawnMode {
    fn default() -> Self {
        SpawnMode::SingleSession
    }
}

// =============================================================================
// WORKER TYPE
// =============================================================================

/// Well-known worker_type values THIS codebase produces. Sent as
/// `metadata.worker_type` at environment registration so claude.ai can filter
/// the session picker by origin (e.g. assistant tab only shows assistant
/// workers). The backend treats this as an opaque string — desktop cowork
/// sends `"cowork"`, which isn't in this union. REPL code uses this narrow
/// type for its own exhaustiveness; wire-level fields accept any string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeWorkerType {
    ClaudeCode,
    ClaudeCodeAssistant,
}

impl BridgeWorkerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BridgeWorkerType::ClaudeCode => "claude_code",
            BridgeWorkerType::ClaudeCodeAssistant => "claude_code_assistant",
        }
    }
}

// =============================================================================
// BRIDGE CONFIG
// =============================================================================

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub dir: String,
    #[serde(rename = "machineName")]
    pub machine_name: String,
    pub branch: String,
    #[serde(rename = "gitRepoUrl")]
    pub git_repo_url: Option<String>,
    #[serde(rename = "maxSessions")]
    pub max_sessions: u32,
    pub spawn_mode: SpawnMode,
    pub verbose: bool,
    pub sandbox: bool,
    /// Client-generated UUID identifying this bridge instance.
    #[serde(rename = "bridgeId")]
    pub bridge_id: String,
    /// Sent as metadata.worker_type so web clients can filter by origin.
    /// Backend treats this as opaque — any string, not just BridgeWorkerType.
    #[serde(rename = "workerType")]
    pub worker_type: String,
    /// Client-generated UUID for idempotent environment registration.
    #[serde(rename = "environmentId")]
    pub environment_id: String,
    /// Backend-issued environment_id to reuse on re-register. When set, the
    /// backend treats registration as a reconnect to the existing environment
    /// instead of creating a new one. Used by `claude remote-control
    /// --session-id` resume. Must be a backend-format ID — client UUIDs are
    /// rejected with 400.
    #[serde(rename = "reuseEnvironmentId")]
    pub reuse_environment_id: Option<String>,
    /// API base URL the bridge is connected to (used for polling).
    #[serde(rename = "apiBaseUrl")]
    pub api_base_url: String,
    /// Session ingress base URL for WebSocket connections (may differ from apiBaseUrl locally).
    #[serde(rename = "sessionIngressUrl")]
    pub session_ingress_url: String,
    /// Debug file path passed via --debug-file.
    #[serde(rename = "debugFile")]
    pub debug_file: Option<String>,
    /// Per-session timeout in milliseconds. Sessions exceeding this are killed.
    #[serde(rename = "sessionTimeoutMs")]
    pub session_timeout_ms: Option<u64>,
}

// =============================================================================
// PERMISSION RESPONSE EVENT
// =============================================================================

/// A control_response event sent back to a session (e.g. a permission decision).
/// The `subtype` is `'success'` per the SDK protocol; the inner `response`
/// carries the permission decision payload (e.g. `{ behavior: 'allow' }`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponseEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub response: PermissionResponseInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponseInner {
    #[serde(rename = "subtype")]
    pub response_subtype: String,
    #[serde(rename = "request_id")]
    pub request_id: String,
    pub response: serde_json::Value,
}

// =============================================================================
// BRIDGE API CLIENT (trait)
// =============================================================================

/// Bridge API client trait for dependency injection
pub trait BridgeApiClient: Send + Sync {
    fn register_bridge_environment(
        &self,
        config: &BridgeConfig,
    ) -> impl std::future::Future<Output = Result<(String, String), String>> + Send;

    fn poll_for_work(
        &self,
        environment_id: &str,
        environment_secret: &str,
        signal: Option<&std::sync::atomic::AtomicBool>,
        reclaim_older_than_ms: Option<u64>,
    ) -> impl std::future::Future<Output = Option<WorkResponse>> + Send;

    fn acknowledge_work(
        &self,
        environment_id: &str,
        work_id: &str,
        session_token: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    /// Stop a work item via the environments API.
    fn stop_work(
        &self,
        environment_id: &str,
        work_id: &str,
        force: bool,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    /// Deregister/delete the bridge environment on graceful shutdown.
    fn deregister_environment(
        &self,
        environment_id: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    /// Send a permission response (control_response) to a session via the session events API.
    fn send_permission_response_event(
        &self,
        session_id: &str,
        event: &PermissionResponseEvent,
        session_token: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    /// Archive a session so it no longer appears as active on the server.
    fn archive_session(
        &self,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    /// Force-stop stale worker instances and re-queue a session on an environment.
    /// Used by `--session-id` to resume a session after the original bridge died.
    fn reconnect_session(
        &self,
        environment_id: &str,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    /// Send a lightweight heartbeat for an active work item, extending its lease.
    /// Uses SessionIngressAuth (JWT, no DB hit) instead of EnvironmentSecretAuth.
    /// Returns the server's response with lease status.
    fn heartbeat_work(
        &self,
        environment_id: &str,
        work_id: &str,
        session_token: &str,
    ) -> impl std::future::Future<Output = Result<HeartbeatResponse, String>> + Send;
}

/// Heartbeat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    #[serde(rename = "lease_extended")]
    pub lease_extended: bool,
    pub state: String,
}

// =============================================================================
// SESSION HANDLE
// =============================================================================

/// Session handle for a running session
pub struct SessionHandle {
    /// Session ID
    pub session_id: String,
    /// Flag indicating if session is done (set by kill/force_kill)
    pub done: bool,
    /// Kill the session gracefully
    pub kill: Box<dyn Fn() + Send + Sync>,
    /// Force kill the session
    pub force_kill: Box<dyn Fn() + Send + Sync>,
    /// Ring buffer of recent activities (last ~10)
    pub activities: Vec<SessionActivity>,
    /// Most recent activity
    pub current_activity: Option<SessionActivity>,
    /// session_ingress_token for API calls
    pub access_token: String,
    /// Ring buffer of last stderr lines
    pub last_stderr: Vec<String>,
    /// Write directly to child stdin
    pub write_stdin: Box<dyn Fn(String) + Send + Sync>,
    /// Update the access token for a running session (e.g. after token refresh).
    pub update_access_token: Box<dyn Fn(String) + Send + Sync>,
}

impl SessionHandle {
    pub fn new(session_id: String, access_token: String) -> Self {
        Self {
            session_id,
            done: false,
            kill: Box::new(|| {}),
            force_kill: Box::new(|| {}),
            activities: Vec::new(),
            current_activity: None,
            access_token,
            last_stderr: Vec::new(),
            write_stdin: Box::new(|_| {}),
            update_access_token: Box::new(|_| {}),
        }
    }
}

// =============================================================================
// SESSION SPAWN OPTS
// =============================================================================

/// Options for spawning a session
pub struct SessionSpawnOpts {
    /// Session ID
    pub session_id: String,
    /// SDK URL
    pub sdk_url: String,
    /// Access token
    pub access_token: String,
    /// When true, spawn the child with CCR v2 env vars (SSE transport + CCRClient).
    pub use_ccr_v2: Option<bool>,
    /// Required when useCcrV2 is true. Obtained from POST /worker/register.
    pub worker_epoch: Option<i64>,
    /// Fires once with the text of the first real user message seen on the
    /// child's stdout (via --replay-user-messages). Lets the caller derive a
    /// session title when none exists yet. Tool-result and synthetic user
    /// messages are skipped.
    pub on_first_user_message: Option<Box<dyn Fn(String) + Send + Sync>>,
}

impl Clone for SessionSpawnOpts {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            sdk_url: self.sdk_url.clone(),
            access_token: self.access_token.clone(),
            use_ccr_v2: self.use_ccr_v2,
            worker_epoch: self.worker_epoch,
            // Callbacks cannot be cloned - set to None
            on_first_user_message: None,
        }
    }
}

// =============================================================================
// SESSION SPAWNER
// =============================================================================

/// Session spawner trait for dependency injection
pub trait SessionSpawner: Send + Sync {
    fn spawn(&self, opts: &SessionSpawnOpts, dir: &str) -> SessionHandle;
}

// =============================================================================
// BRIDGE LOGGER
// =============================================================================

/// Bridge logger trait for displaying status
pub trait BridgeLogger: Send + Sync {
    /// Print banner with configuration
    fn print_banner(&self, config: &BridgeConfig, environment_id: &str);

    /// Log session start
    fn log_session_start(&self, session_id: &str, prompt: &str);

    /// Log session complete
    fn log_session_complete(&self, session_id: &str, duration_ms: u64);

    /// Log session failed
    fn log_session_failed(&self, session_id: &str, error: &str);

    /// Log status message
    fn log_status(&self, message: &str);

    /// Log verbose message
    fn log_verbose(&self, message: &str);

    /// Log error message
    fn log_error(&self, message: &str);

    /// Log reconnection success
    fn log_reconnected(&self, disconnected_ms: u64);

    /// Set repository info for status line display
    fn set_repo_info(&self, repo_name: &str, branch: &str);

    /// Set debug log path shown above the status line (ant users)
    fn set_debug_log_path(&self, path: &str);

    /// Show idle status with repo/branch info and shimmer animation
    fn update_idle_status(&self);

    /// Transition to "Attached" state when a session starts
    fn set_attached(&self, session_id: &str);

    /// Show reconnecting status in the live display
    fn update_reconnecting_status(&self, delay_str: &str, elapsed_str: &str);

    /// Update session status
    fn update_session_status(
        &self,
        session_id: &str,
        elapsed: &str,
        activity: &SessionActivity,
        trail: &[String],
    );

    /// Clear status
    fn clear_status(&self);

    /// Toggle QR code visibility
    fn toggle_qr(&self);

    /// Update the "<n> of <m> sessions" indicator and spawn mode hint
    fn update_session_count(&self, active: u32, max: u32, mode: SpawnMode);

    /// Update the spawn mode shown in the session-count line
    fn set_spawn_mode_display(&self, mode: Option<SpawnMode>);

    /// Register a new session for multi-session display
    fn add_session(&self, session_id: &str, url: &str);

    /// Update the per-session activity summary in the multi-session list
    fn update_session_activity(&self, session_id: &str, activity: &SessionActivity);

    /// Set a session's display title
    fn set_session_title(&self, session_id: &str, title: &str);

    /// Remove a session from the multi-session display when it ends
    fn remove_session(&self, session_id: &str);

    /// Force a re-render of the status display
    fn refresh_display(&self);
}
