// Source: ~/claudecode/openclaudecode/src/utils/statusNoticeHelpers.rs

/// Threshold for agent descriptions token count.
pub const AGENT_DESCRIPTIONS_THRESHOLD: usize = 15_000;

/// Calculate cumulative token estimate for agent descriptions.
pub fn get_agent_descriptions_total_tokens(
    agent_definitions: Option<&AgentDefinitionsResult>,
) -> usize {
    let Some(defs) = agent_definitions else {
        return 0;
    };

    defs.active_agents
        .iter()
        .filter(|a| a.source != "built-in")
        .map(|agent| {
            let description = format!("{}: {}", agent.agent_type, agent.when_to_use);
            rough_token_count_estimation(&description)
        })
        .sum()
}

/// Rough token count estimation (1 token ~ 4 chars for English text).
fn rough_token_count_estimation(text: &str) -> usize {
    text.len().div_ceil(4)
}

/// Result of loading agent definitions.
pub struct AgentDefinitionsResult {
    pub active_agents: Vec<AgentDefinition>,
}

/// An agent definition.
pub struct AgentDefinition {
    pub agent_type: String,
    pub when_to_use: String,
    pub source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_returns_zero() {
        assert_eq!(get_agent_descriptions_total_tokens(None), 0);
    }

    #[test]
    fn test_empty_agents_returns_zero() {
        let defs = AgentDefinitionsResult {
            active_agents: vec![],
        };
        assert_eq!(get_agent_descriptions_total_tokens(Some(&defs)), 0);
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(rough_token_count_estimation("hello world"), 3);
        assert_eq!(rough_token_count_estimation(""), 0);
    }
}
