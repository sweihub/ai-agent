//! Swarm types
//!
//! Translates from TypeScript: src/utils/swarm/backends/types.ts

/// Agent color names for UI differentiation
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentColorName {
    Red,
    Blue,
    Green,
    Yellow,
    Cyan,
    Magenta,
    White,
    Black,
}

impl Default for AgentColorName {
    fn default() -> Self {
        Self::Blue
    }
}

impl std::fmt::Display for AgentColorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentColorName::Red => write!(f, "red"),
            AgentColorName::Blue => write!(f, "blue"),
            AgentColorName::Green => write!(f, "green"),
            AgentColorName::Yellow => write!(f, "yellow"),
            AgentColorName::Cyan => write!(f, "cyan"),
            AgentColorName::Magenta => write!(f, "magenta"),
            AgentColorName::White => write!(f, "white"),
            AgentColorName::Black => write!(f, "black"),
        }
    }
}

/// Types of backends available for teammate execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendType {
    /// Uses tmux for pane management (works in tmux or standalone)
    Tmux,
    /// Uses iTerm2 native split panes via the it2 CLI
    Iterm2,
    /// Runs teammate in the same process with isolated context
    InProcess,
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::Tmux => write!(f, "tmux"),
            BackendType::Iterm2 => write!(f, "iterm2"),
            BackendType::InProcess => write!(f, "in-process"),
        }
    }
}

/// Subset of BackendType for pane-based backends only.
/// Used in messages and types that specifically deal with terminal panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaneBackendType {
    Tmux,
    Iterm2,
}

impl From<PaneBackendType> for BackendType {
    fn from(p: PaneBackendType) -> Self {
        match p {
            PaneBackendType::Tmux => BackendType::Tmux,
            PaneBackendType::Iterm2 => BackendType::Iterm2,
        }
    }
}

/// Opaque identifier for a pane managed by a backend.
/// For tmux, this is the tmux pane ID (e.g., "%1").
/// For iTerm2, this is the session ID returned by it2.
pub type PaneId = String;

/// Result of creating a new teammate pane.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatePaneResult {
    /// The pane ID for the newly created pane
    pub pane_id: PaneId,
    /// Whether this is the first teammate pane (affects layout strategy)
    pub is_first_teammate: bool,
}

/// Identity fields for a teammate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeammateIdentity {
    /// Agent name (e.g., "researcher", "tester")
    pub name: String,
    /// Team name this teammate belongs to
    pub team_name: String,
    /// Assigned color for UI differentiation
    #[serde(default)]
    pub color: Option<AgentColorName>,
    /// Whether plan mode approval is required before implementation
    #[serde(default)]
    pub plan_mode_required: Option<bool>,
}

/// System prompt mode for teammate
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemPromptMode {
    Default,
    Replace,
    Append,
}

impl Default for SystemPromptMode {
    fn default() -> Self {
        Self::Default
    }
}

/// Configuration for spawning a teammate (any execution mode).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeammateSpawnConfig {
    /// Agent name
    pub name: String,
    /// Team name this teammate belongs to
    pub team_name: String,
    /// Assigned color for UI differentiation
    #[serde(default)]
    pub color: Option<AgentColorName>,
    /// Whether plan mode approval is required before implementation
    #[serde(default)]
    pub plan_mode_required: Option<bool>,
    /// Initial prompt to send to the teammate
    pub prompt: String,
    /// Working directory for the teammate
    pub cwd: String,
    /// Model to use for this teammate
    #[serde(default)]
    pub model: Option<String>,
    /// System prompt for this teammate (resolved from workflow config)
    #[serde(default)]
    pub system_prompt: Option<String>,
    /// How to apply the system prompt: 'default', 'replace' or 'append' to default
    #[serde(default)]
    pub system_prompt_mode: Option<SystemPromptMode>,
    /// Optional git worktree path
    #[serde(default)]
    pub worktree_path: Option<String>,
    /// Parent session ID (for context linking)
    pub parent_session_id: String,
    /// Tool permissions to grant this teammate
    #[serde(default)]
    pub permissions: Option<Vec<String>>,
    /// Whether this teammate can show permission prompts for unlisted tools.
    /// When false (default), unlisted tools are auto-denied.
    #[serde(default)]
    pub allow_permission_prompts: Option<bool>,
}

/// Result from spawning a teammate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeammateSpawnResult {
    /// Whether spawn was successful
    pub success: bool,
    /// Unique agent ID (format: agentName@teamName)
    pub agent_id: String,
    /// Error message if spawn failed
    #[serde(default)]
    pub error: Option<String>,
    /// Task ID in AppState.tasks (in-process only).
    /// Used for UI rendering and progress tracking.
    /// agentId is the logical identifier; taskId is for AppState indexing.
    #[serde(default)]
    pub task_id: Option<String>,
    /// Pane ID (pane-based only)
    #[serde(default)]
    pub pane_id: Option<PaneId>,
}

/// Message to send to a teammate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeammateMessage {
    /// Message content
    pub text: String,
    /// Sender agent ID
    pub from: String,
    /// Sender display color
    #[serde(default)]
    pub color: Option<String>,
    /// Message timestamp (ISO string)
    #[serde(default)]
    pub timestamp: Option<String>,
    /// 5-10 word summary shown as preview in the UI
    #[serde(default)]
    pub summary: Option<String>,
}

/// Type guard to check if a backend type uses terminal panes.
pub fn is_pane_backend(backend_type: &BackendType) -> bool {
    matches!(backend_type, BackendType::Tmux | BackendType::Iterm2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_color_names() {
        let colors = [
            AgentColorName::Red,
            AgentColorName::Blue,
            AgentColorName::Green,
            AgentColorName::Yellow,
            AgentColorName::Cyan,
            AgentColorName::Magenta,
            AgentColorName::White,
            AgentColorName::Black,
        ];

        for color in colors {
            let s = color.to_string();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_backend_type_display() {
        assert_eq!(BackendType::Tmux.to_string(), "tmux");
        assert_eq!(BackendType::Iterm2.to_string(), "iterm2");
        assert_eq!(BackendType::InProcess.to_string(), "in-process");
    }

    #[test]
    fn test_is_pane_backend() {
        assert!(is_pane_backend(&BackendType::Tmux));
        assert!(is_pane_backend(&BackendType::Iterm2));
        assert!(!is_pane_backend(&BackendType::InProcess));
    }

    #[test]
    fn test_teammate_spawn_config() {
        let config = TeammateSpawnConfig {
            name: "tester".to_string(),
            team_name: "my-team".to_string(),
            color: Some(AgentColorName::Green),
            plan_mode_required: Some(true),
            prompt: "Run tests".to_string(),
            cwd: "/tmp".to_string(),
            model: Some("claude-3-opus".to_string()),
            system_prompt: Some("You are a tester".to_string()),
            system_prompt_mode: Some(SystemPromptMode::Append),
            worktree_path: None,
            parent_session_id: "parent-123".to_string(),
            permissions: Some(vec!["tool1".to_string(), "tool2".to_string()]),
            allow_permission_prompts: Some(true),
        };

        assert_eq!(config.name, "tester");
        assert_eq!(config.team_name, "my-team");
        assert_eq!(config.color, Some(AgentColorName::Green));
    }

    #[test]
    fn test_teammate_message_serialization() {
        let msg = TeammateMessage {
            text: "Hello from teammate".to_string(),
            from: "tester@my-team".to_string(),
            color: Some("#FF0000".to_string()),
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
            summary: Some("Greeting message".to_string()),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Hello from teammate"));
    }

    #[test]
    fn test_system_prompt_mode_default() {
        let mode: SystemPromptMode = serde_json::from_str("\"default\"").unwrap();
        assert_eq!(mode, SystemPromptMode::Default);

        let mode: SystemPromptMode = serde_json::from_str("\"replace\"").unwrap();
        assert_eq!(mode, SystemPromptMode::Replace);

        let mode: SystemPromptMode = serde_json::from_str("\"append\"").unwrap();
        assert_eq!(mode, SystemPromptMode::Append);
    }
}
