// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/builtInAgents.ts
use super::AgentDefinition;
use super::built_in_dir::{
    claude_code_guide_agent, explore_agent, general_purpose_agent, plan_agent, statusline_setup,
    verification_agent,
};

/// Check if explore/plan built-in agents are enabled.
pub fn are_explore_plan_agents_enabled() -> bool {
    std::env::var("AI_CODE_ENABLE_EXPLORE_PLAN_AGENTS")
        .map(|v| v != "0" && v != "false" && v != "no")
        .unwrap_or(true)
}

/// Get the list of built-in agents.
/// Respects the AI_CODE_DISABLE_BUILTIN_AGENTS env var.
pub fn get_built_in_agents() -> Vec<AgentDefinition> {
    // Allow disabling all built-in agents via env var
    let disable_builtin = std::env::var("AI_CODE_DISABLE_BUILTIN_AGENTS")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false);

    let non_interactive = std::env::var("AI_CODE_NON_INTERACTIVE")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false);

    if disable_builtin && non_interactive {
        return vec![];
    }

    // Check for coordinator mode
    let coordinator_mode = std::env::var("AI_CODE_COORDINATOR_MODE")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false);

    if coordinator_mode {
        // Coordinator mode uses a minimal set of agents
        return vec![general_purpose_agent::general_purpose_agent()];
    }

    let mut agents: Vec<AgentDefinition> = vec![
        general_purpose_agent::general_purpose_agent(),
        statusline_setup::statusline_setup_agent(),
    ];

    if are_explore_plan_agents_enabled() {
        agents.push(explore_agent::explore_agent());
        agents.push(plan_agent::plan_agent());
    }

    // Include Code Guide agent for non-SDK entrypoints
    let entrypoint = std::env::var("AI_CODE_ENTRYPOINT").unwrap_or_default();
    let is_non_sdk_entrypoint =
        entrypoint != "sdk-ts" && entrypoint != "sdk-py" && entrypoint != "sdk-cli";

    if is_non_sdk_entrypoint {
        agents.push(claude_code_guide_agent::claude_code_guide_agent());
    }

    // Verification agent feature flag check
    let verification_enabled = std::env::var("AI_CODE_ENABLE_VERIFICATION_AGENT")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false);

    if verification_enabled {
        agents.push(verification_agent::verification_agent());
    }

    agents
}

/// Get the general-purpose agent as a fallback.
pub fn general_purpose_agent() -> AgentDefinition {
    general_purpose_agent::general_purpose_agent()
}
