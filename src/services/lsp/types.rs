// Source: /data/home/swei/claudecode/openclaudecode/src/services/lsp/types.ts

/// LSP server configuration scoped to a workspace
pub type ScopedLspServerConfig = std::collections::HashMap<String, serde_json::Value>;

/// LSP server state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspServerState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl Default for LspServerState {
    fn default() -> Self {
        LspServerState::Stopped
    }
}

impl std::fmt::Display for LspServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LspServerState::Stopped => write!(f, "stopped"),
            LspServerState::Starting => write!(f, "starting"),
            LspServerState::Running => write!(f, "running"),
            LspServerState::Stopping => write!(f, "stopping"),
            LspServerState::Error => write!(f, "error"),
        }
    }
}

impl From<&str> for LspServerState {
    fn from(s: &str) -> Self {
        match s {
            "stopped" => LspServerState::Stopped,
            "starting" => LspServerState::Starting,
            "running" => LspServerState::Running,
            "stopping" => LspServerState::Stopping,
            "error" => LspServerState::Error,
            _ => LspServerState::Stopped,
        }
    }
}
