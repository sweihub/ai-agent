// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/agentDisplay.ts
#![allow(dead_code)]
use std::sync::Arc;

use std::collections::HashMap;

use super::load_agents_dir::AgentDefinition;

/// Represents a source group for display ordering.
#[derive(Debug, Clone)]
pub struct AgentSourceGroup {
    pub label: &'static str,
    pub source: &'static str,
}

/// Ordered list of agent source groups for display.
pub const AGENT_SOURCE_GROUPS: &[AgentSourceGroup] = &[
    AgentSourceGroup { label: "User agents", source: "userSettings" },
    AgentSourceGroup { label: "Project agents", source: "projectSettings" },
    AgentSourceGroup { label: "Local agents", source: "localSettings" },
    AgentSourceGroup { label: "Managed agents", source: "policySettings" },
    AgentSourceGroup { label: "Plugin agents", source: "plugin" },
    AgentSourceGroup { label: "CLI arg agents", source: "flagSettings" },
    AgentSourceGroup { label: "Built-in agents", source: "built-in" },
];

/// Agent definition with override information.
#[derive(Debug, Clone)]
pub struct ResolvedAgent {
    pub definition: AgentDefinition,
    pub overridden_by: Option<String>,
}

/// Annotate agents with override information by comparing against the active
/// (winning) agent list. An agent is "overridden" when another agent with the
/// same type from a higher-priority source takes precedence.
///
/// Also deduplicates by (agent_type, source) to handle git worktree duplicates.
pub fn resolve_agent_overrides(
    all_agents: Vec<AgentDefinition>,
    active_agents: &[AgentDefinition],
) -> Vec<ResolvedAgent> {
    let mut active_map: HashMap<String, &AgentDefinition> = HashMap::new();
    for agent in active_agents {
        active_map.insert(agent.agent_type.clone(), agent);
    }

    let mut seen: HashMap<String, bool> = HashMap::new();
    let mut resolved: Vec<ResolvedAgent> = Vec::new();

    for agent in all_agents {
        let key = format!("{}:{}", agent.agent_type, agent.source);
        if seen.contains_key(&key) {
            continue;
        }
        seen.insert(key, true);

        let overridden_by = active_map.get(&agent.agent_type).and_then(|active| {
            if active.source != agent.source {
                Some(active.source.clone())
            } else {
                None
            }
        });

        resolved.push(ResolvedAgent {
            definition: agent,
            overridden_by,
        });
    }

    resolved
}

/// Resolve the display model string for an agent.
pub fn resolve_agent_model_display(agent: &AgentDefinition) -> Option<String> {
    let model = agent.model.as_deref().unwrap_or_else(|| get_default_subagent_model().unwrap_or("sonnet"));
    if model.is_empty() {
        return None;
    }
    Some(if model == "inherit" {
        "inherit".to_string()
    } else {
        model.to_string()
    })
}

/// Get the default subagent model.
fn get_default_subagent_model() -> Option<&'static str> {
    Some("sonnet")
}

/// Get a human-readable label for the source that overrides an agent.
pub fn get_override_source_label(source: &str) -> String {
    get_source_display_name(source).to_lowercase()
}

/// Get the display name for a source.
fn get_source_display_name(source: &str) -> &str {
    match source {
        "userSettings" => "User",
        "projectSettings" => "Project",
        "localSettings" => "Local",
        "policySettings" => "Managed",
        "plugin" => "Plugin",
        "flagSettings" => "CLI",
        "built-in" => "Built-in",
        _ => source,
    }
}

/// Compare agents alphabetically by name (case-insensitive).
pub fn compare_agents_by_name(a: &AgentDefinition, b: &AgentDefinition) -> std::cmp::Ordering {
    a.agent_type
        .to_lowercase()
        .cmp(&b.agent_type.to_lowercase())
}

/// Format one agent line for display.
pub fn format_agent_line(agent: &AgentDefinition) -> String {
    let tools_description = get_tools_description(agent);
    format!(
        "- {}: {} (Tools: {})",
        agent.agent_type, agent.when_to_use, tools_description
    )
}

/// Get a description of the tools available to an agent.
fn get_tools_description(agent: &AgentDefinition) -> String {
    let has_allowlist = agent.tools.iter().any(|t| !t.starts_with('-'));
    let denylist: Vec<&String> = agent
        .disallowed_tools
        .iter()
        .filter(|t| t.starts_with('-'))
        .collect();
    let has_denylist = !denylist.is_empty();

    if has_allowlist && has_denylist {
        // Both defined: filter allowlist by denylist to match runtime behavior
        let deny_set: std::collections::HashSet<&str> =
            denylist.iter().map(|t| t.as_str()).collect();
        let effective: Vec<&String> = agent
            .tools
            .iter()
            .filter(|t| !deny_set.contains(t.as_str()))
            .collect();
        if effective.is_empty() {
            return "None".to_string();
        }
        effective
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    } else if has_allowlist {
        // Allowlist only: show the specific tools available
        agent.tools.join(", ")
    } else if has_denylist {
        // Denylist only: show "All tools except X, Y, Z"
        let tools: Vec<&str> = denylist.iter().map(|t| t.as_str()).collect();
        format!("All tools except {}", tools.join(", "))
    } else {
        // No restrictions
        "All tools".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(agent_type: &str, source: &str) -> AgentDefinition {
        AgentDefinition {
            agent_type: agent_type.to_string(),
            when_to_use: "test".to_string(),
            tools: vec!["*".to_string()],
            source: source.to_string(),
            base_dir: "built-in".to_string(),
            get_system_prompt: Arc::new(|| String::new()),
            model: None,
            disallowed_tools: vec![],
            max_turns: None,
            permission_mode: None,
            effort: None,
            color: None,
            mcp_servers: vec![],
            hooks: None,
            skills: vec![],
            background: false,
            initial_prompt: None,
            memory: None,
            isolation: None,
            required_mcp_servers: vec![],
            omit_claude_md: false,
            critical_system_reminder_experimental: None,
        }
    }

    #[test]
    fn test_resolve_overrides() {
        let all_agents = vec![
            make_agent("test", "built-in"),
            make_agent("test", "userSettings"),
        ];
        let active = vec![make_agent("test", "userSettings")];
        let resolved = resolve_agent_overrides(all_agents, &active);
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_compare_agents_by_name() {
        let a = make_agent("Beta", "built-in");
        let b = make_agent("alpha", "built-in");
        assert_eq!(compare_agents_by_name(&a, &b), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_resolve_agent_model_display_inherit() {
        let agent = make_agent("test", "built-in");
        assert_eq!(
            resolve_agent_model_display(&agent),
            Some("sonnet".to_string())
        );
    }
}
