// Source: /data/home/swei/claudecode/openclaudecode/src/tools/SkillTool/constants.ts
//! Swarm-related constants for teammate sessions.

pub const TEAM_LEAD_NAME: &str = "team-lead";
pub const SWARM_SESSION_NAME: &str = "claude-swarm";
pub const SWARM_VIEW_WINDOW_NAME: &str = "swarm-view";
pub const TMUX_COMMAND: &str = "tmux";
pub const HIDDEN_SESSION_NAME: &str = "claude-hidden";

/// Gets the socket name for external swarm sessions (when user is not in tmux).
/// Uses a separate socket to isolate swarm operations from user's tmux sessions.
/// Includes PID to ensure multiple Claude instances don't conflict.
pub fn get_swarm_socket_name() -> String {
    format!("claude-swarm-{}", std::process::id())
}

/// Environment variable to override the command used to spawn teammate instances.
/// If not set, defaults to process.execPath (the current Claude binary).
/// This allows customization for different environments or testing.
pub const TEAMMATE_COMMAND_ENV_VAR: &str = "AI_CODE_TEAMMATE_COMMAND";

/// Environment variable set on spawned teammates to indicate their assigned color.
/// Used for colored output and pane identification.
pub const TEAMMATE_COLOR_ENV_VAR: &str = "AI_CODE_AGENT_COLOR";

/// Environment variable set on spawned teammates to require plan mode before implementation.
/// When set to 'true', teammates must enter plan mode and get approval before writing code.
pub const PLAN_MODE_REQUIRED_ENV_VAR: &str = "AI_CODE_PLAN_MODE_REQUIRED";
