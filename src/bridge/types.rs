// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
//! Bridge types for remote control functionality

use std::collections::HashMap;

/// Default per-session timeout (24 hours)
pub const DEFAULT_SESSION_TIMEOUT_MS: u64 = 24 * 60 * 60 * 1000;

pub const BRIDGE_LOGIN_INSTRUCTION: &str =
    "Remote Control is only available with claude.ai subscriptions. Please use `/login` to sign in with your claude.ai account.";

pub const BRIDGE_LOGIN_ERROR: &str = "Error: You must be logged in to use Remote Control.\n\n\
    Remote Control is only available with claude.ai subscriptions. Please use `/login` to sign in with your claude.ai account.";

pub const REMOTE_CONTROL_DISCONNECTED_MSG: &str = "Remote Control disconnected.";

#[derive(Debug, Clone)]
pub enum WorkDataType {
    Session,
    Healthcheck,
}

#[derive(Debug, Clone)]
pub struct WorkData {
    pub work_type: WorkDataType,
    pub id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub environment_id: String,
    pub state: String,
    pub data: WorkData,
    pub secret: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitInfo {
    pub git_type: String,
    pub repo: String,
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthEntry {
    pub auth_type: String,
    pub token: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SourceEntry {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(rename = "git_info")]
    pub git_info: Option<GitInfo>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkSecret {
    pub version: u32,
    #[serde(rename = "session_ingress_token")]
    pub session_ingress_token: String,
    #[serde(rename = "api_base_url")]
    pub api_base_url: String,
    pub sources: Vec<SourceEntry>,
    pub auth: Vec<AuthEntry>,
    #[serde(rename = "claude_code_args")]
    pub claude_code_args: Option<HashMap<String, String>>,
    #[serde(rename = "mcp_config")]
    pub mcp_config: Option<serde_json::Value>,
    #[serde(rename = "environment_variables")]
    pub environment_variables: Option<HashMap<String, String>>,
    #[serde(rename = "use_code_sessions")]
    pub use_code_sessions: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionDoneStatus {
    Completed,
    Failed,
    Interrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionActivityType {
    ToolStart,
    Text,
    Result,
    Error,
}

#[derive(Debug, Clone)]
pub struct SessionActivity {
    pub activity_type: SessionActivityType,
    pub summary: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnMode {
    SingleSession,
    Worktree,
    SameDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeWorkerType {
    ClaudeCode,
    ClaudeCodeAssistant,
}

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub dir: String,
    pub machine_name: String,
    pub branch: String,
    pub git_repo_url: Option<String>,
    pub max_sessions: u32,
    pub spawn_mode: SpawnMode,
    pub verbose: bool,
    pub sandbox: bool,
    pub bridge_id: String,
    pub worker_type: String,
    pub environment_id: String,
    pub reuse_environment_id: Option<String>,
    pub api_base_url: String,
    pub session_ingress_url: String,
    pub debug_file: Option<String>,
    pub session_timeout_ms: Option<u64>,
}
