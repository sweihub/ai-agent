// Source: ~/claudecode/openclaudecode/src/utils/promptCategory.ts

use serde::Serialize;

/// Query source for analytics tracking.
#[derive(Debug, Clone, Serialize)]
pub enum QuerySource {
    #[serde(rename = "agent:default")]
    AgentDefault,
    #[serde(rename = "agent:custom")]
    AgentCustom,
    #[serde(rename = "repl_main_thread")]
    ReplMainThread,
}

/// Determines the prompt category for agent usage.
/// Used for analytics to track different agent patterns.
pub fn get_query_source_for_agent(
    agent_type: Option<&str>,
    is_built_in_agent: bool,
) -> QuerySource {
    if is_built_in_agent {
        match agent_type {
            Some(t) => {
                // Build a dynamic variant - in production this would map to
                // specific query source strings
                QuerySource::AgentDefault
            }
            None => QuerySource::AgentDefault,
        }
    } else {
        QuerySource::AgentCustom
    }
}

/// Determines the prompt category based on output style settings.
/// Used for analytics to track different output style usage.
pub fn get_query_source_for_repl() -> QuerySource {
    // Check for output style setting (localized env var)
    let style = std::env::var("AI_CODE_OUTPUT_STYLE")
        .ok()
        .unwrap_or_else(|| "default".to_string());

    if style == "default" {
        return QuerySource::ReplMainThread;
    }

    // All built-in styles get their name in the query source
    // In production, this would check against OUTPUT_STYLE_CONFIG
    let is_built_in = matches!(
        style.as_str(),
        "compact" | "verbose" | "minimal" | "default"
    );

    if is_built_in {
        QuerySource::ReplMainThread
    } else {
        QuerySource::ReplMainThread
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_query_source_for_agent_builtin() {
        let source = get_query_source_for_agent(Some("planner"), true);
        assert!(matches!(source, QuerySource::AgentDefault));
    }

    #[test]
    fn test_get_query_source_for_agent_custom() {
        let source = get_query_source_for_agent(None, false);
        assert!(matches!(source, QuerySource::AgentCustom));
    }

    #[test]
    fn test_get_query_source_for_repl_default() {
        std::env::remove_var("AI_CODE_OUTPUT_STYLE");
        let source = get_query_source_for_repl();
        assert!(matches!(source, QuerySource::ReplMainThread));
    }
}
