//! Plan mode v2 utilities.

use crate::constants::env::ai;
use serde::{Deserialize, Serialize};

/// Plan mode v2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanModeV2Config {
    pub enabled: bool,
    pub use_interview: bool,
    pub agent_count: usize,
    pub explore_agent_count: usize,
}

impl Default for PlanModeV2Config {
    fn default() -> Self {
        Self {
            enabled: false,
            use_interview: false,
            agent_count: 1,
            explore_agent_count: 1,
        }
    }
}

/// Check if plan mode v2 is enabled
pub fn is_plan_mode_v2_enabled() -> bool {
    std::env::var(ai::CODE_PLAN_MODE_V2)
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Get the agent count for plan mode v2
pub fn get_plan_mode_v2_agent_count() -> usize {
    std::env::var(ai::CODE_PLAN_MODE_AGENT_COUNT)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

/// Get the explore agent count for plan mode v2
pub fn get_plan_mode_v2_explore_agent_count() -> usize {
    std::env::var(ai::CODE_PLAN_MODE_EXPLORE_AGENT_COUNT)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

/// Check if interview phase is enabled in plan mode v2
pub fn is_plan_mode_interview_phase_enabled() -> bool {
    std::env::var(ai::CODE_PLAN_MODE_INTERVIEW)
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Get the Pewter ledger variant
pub fn get_pewter_ledger_variant() -> Option<String> {
    std::env::var(ai::CODE_PEWTER_LEDGER_VARIANT).ok()
}
