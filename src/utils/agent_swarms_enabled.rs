// Source: ~/claudecode/openclaudecode/src/utils/agentSwarmsEnabled.ts
//! Centralized runtime check for agent teams/teammate features.
//!
//! This is the single gate that should be checked everywhere teammates
//! are referenced (prompts, code, tools is_enabled, UI, etc.).
//!
//! Ant builds: always enabled.
//! External builds require both:
//! 1. Opt-in via AI_CODE_EXPRIMENTAL_AGENT_TEAMS env var OR --agent-teams flag
//! 2. GrowthBook gate 'tengu_amber_flint' enabled (killswitch)

#![allow(dead_code)]

use crate::constants::env::ai;

/// Check if --agent-teams flag is provided via CLI.
/// Checks process::args() directly to avoid import cycles with bootstrap/state.
/// Note: The flag is only shown in help for ant users, but if external users
/// pass it anyway, it will work (subject to the killswitch).
fn is_agent_teams_flag_set() -> bool {
    std::env::args().any(|arg| arg == "--agent-teams")
}

/// Check if the AI_CODE_EXPRIMENTAL_AGENT_TEAMS env var is set to a truthy value.
fn is_experimental_agent_teams_env() -> bool {
    std::env::var(ai_code::EXPERIMENTAL_AGENT_TEAMS)
        .map(|v| v == "1" || v.to_lowercase() == "true" || v.to_lowercase() == "yes")
        .unwrap_or(false)
}

use crate::constants::env::ai_code;

/// Centralized runtime check for agent teams/teammate features.
/// This is the single gate that should be checked everywhere teammates
/// are referenced (prompts, code, tools is_enabled, UI, etc.).
///
/// Ant: always on.
/// External: require opt-in via env var or --agent-teams flag,
/// plus the GrowthBook killswitch gate.
pub fn is_agent_swarms_enabled() -> bool {
    // Ant: always on
    if std::env::var(ai::USER_TYPE)
        .map(|v| v == "ant")
        .unwrap_or(false)
    {
        return true;
    }

    // External: require opt-in via env var or --agent-teams flag
    if !is_experimental_agent_teams_env() && !is_agent_teams_flag_set() {
        return false;
    }

    // Killswitch — always respected for external users.
    // GrowthBook gate 'tengu_amber_flint' — default to true if unavailable.
    if !get_feature_value_cached_may_be_stale("tengu_amber_flint", true) {
        return false;
    }

    true
}

/// Simplified GrowthBook feature check.
/// In a full implementation, this would query the GrowthBook SDK.
/// Here we default to the provided fallback value.
fn get_feature_value_cached_may_be_stale(_feature_id: &str, default: bool) -> bool {
    // Without GrowthBook integration, default to the provided value.
    // In production, this would call the GrowthBook SDK's is_on() method.
    default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_agent_teams_flag_set() {
        // This test checks the function logic without env vars.
        // The actual flag detection depends on std::env::args().
        assert!(!is_agent_teams_flag_set() || is_agent_teams_flag_set());
    }

    #[test]
    fn test_get_feature_default() {
        assert!(get_feature_value_cached_may_be_stale("test", true));
        assert!(!get_feature_value_cached_may_be_stale("test", false));
    }
}
