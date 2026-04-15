//! Swarm constants
//!
//! Translates from TypeScript: src/utils/swarm/constants.ts

/// Team lead agent name
pub const TEAM_LEAD_NAME: &str = "team-lead";

/// Name for swarm sessions
pub const SWARM_SESSION_NAME: &str = "claude-swarm";

/// Window name for swarm view
pub const SWARM_VIEW_WINDOW_NAME: &str = "swarm-view";

/// Default tmux command
pub const TMUX_COMMAND: &str = "tmux";

/// Hidden session name
pub const HIDDEN_SESSION_NAME: &str = "claude-hidden";

/// Environment variable to override the command used to spawn teammate instances.
/// If not set, defaults to the current executable.
/// This allows customization for different environments or testing.
pub const TEAMMATE_COMMAND_ENV_VAR: &str = "AI_TEAMMATE_COMMAND";

/// Environment variable set on spawned teammates to indicate their assigned color.
/// Used for colored output and pane identification.
pub const TEAMMATE_COLOR_ENV_VAR: &str = "AI_AGENT_COLOR";

/// Environment variable set on spawned teammates to require plan mode before implementation.
/// When set to 'true', teammates must enter plan mode and get approval before writing code.
pub const PLAN_MODE_REQUIRED_ENV_VAR: &str = "AI_PLAN_MODE_REQUIRED";

/// Gets the socket name for external swarm sessions (when user is not in tmux).
/// Uses a separate socket to isolate swarm operations from user's tmux sessions.
/// Includes PID to ensure multiple Claude instances don't conflict.
pub fn get_swarm_socket_name() -> String {
    format!("claude-swarm-{}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(TEAM_LEAD_NAME, "team-lead");
        assert_eq!(SWARM_SESSION_NAME, "claude-swarm");
        assert_eq!(SWARM_VIEW_WINDOW_NAME, "swarm-view");
        assert_eq!(TMUX_COMMAND, "tmux");
        assert_eq!(HIDDEN_SESSION_NAME, "claude-hidden");
    }

    #[test]
    fn test_env_vars_localized() {
        // Check that env vars are localized to AI_ prefix
        assert!(TEAMMATE_COMMAND_ENV_VAR.starts_with("AI_"));
        assert!(TEAMMATE_COLOR_ENV_VAR.starts_with("AI_"));
        assert!(PLAN_MODE_REQUIRED_ENV_VAR.starts_with("AI_"));
    }

    #[test]
    fn test_get_swarm_socket_name() {
        let socket_name = get_swarm_socket_name();
        assert!(socket_name.starts_with("claude-swarm-"));
        // PID should be non-zero
        assert!(socket_name.contains(&std::process::id().to_string()));
    }
}
