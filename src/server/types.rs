// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
use serde::{Deserialize, Serialize};

pub const CONNECT_RESPONSE_SCHEMA: &str = r#"{
  "session_id": "string",
  "ws_url": "string",
  "work_dir": "string"
}"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponse {
    pub session_id: String,
    pub ws_url: String,
    #[serde(default)]
    pub work_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub auth_token: String,
    #[serde(default)]
    pub unix: Option<String>,
    #[serde(rename = "idleTimeoutMs", default)]
    pub idle_timeout_ms: Option<u64>,
    #[serde(rename = "maxSessions", default)]
    pub max_sessions: Option<usize>,
    #[serde(default)]
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Starting,
    Running,
    Detached,
    Stopping,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: String,
    pub status: SessionState,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "workDir")]
    pub work_dir: String,
    #[serde(default)]
    pub process: Option<()>,
    #[serde(rename = "sessionKey", default)]
    pub session_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIndexEntry {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "transcriptSessionId")]
    pub transcript_session_id: String,
    pub cwd: String,
    #[serde(default)]
    pub permission_mode: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "lastActiveAt")]
    pub last_active_at: i64,
}

pub type SessionIndex = std::collections::HashMap<String, SessionIndexEntry>;
