//! Swarm utilities
//!
//! Provides types and constants for swarm mode (multi-agent collaboration).

pub mod swarm_constants;
pub mod swarm_types;

pub use swarm_constants::{
    get_swarm_socket_name, HIDDEN_SESSION_NAME, PLAN_MODE_REQUIRED_ENV_VAR, SWARM_SESSION_NAME,
    SWARM_VIEW_WINDOW_NAME, TEAMMATE_COLOR_ENV_VAR, TEAMMATE_COMMAND_ENV_VAR, TEAM_LEAD_NAME,
    TMUX_COMMAND,
};

pub use swarm_types::{
    is_pane_backend, AgentColorName, BackendType, CreatePaneResult, PaneBackendType, PaneId,
    SystemPromptMode, TeammateIdentity, TeammateMessage, TeammateSpawnConfig, TeammateSpawnResult,
};
