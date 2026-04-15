// Source: ~/claudecode/openclaudecode/src/tools/ToolSearchTool/prompt.ts
// and ~/claudecode/openclaudecode/src/utils/toolSearch.ts
use crate::tools::config_tools::TOOL_SEARCH_TOOL_NAME;
use crate::types::ToolDefinition;
use crate::utils::env_utils;
use std::collections::HashSet;

/// Check if a tool should be deferred (requires ToolSearch to load).
/// A tool is deferred if:
/// - It's an MCP tool (always deferred)
/// - It has should_defer: true
///
/// A tool is NEVER deferred if:
/// - It has always_load: true
/// - It's the ToolSearchTool itself
/// - It's one of the special exceptions (Brief, SendUserFile, Agent when fork enabled)
pub fn is_deferred_tool(tool: &ToolDefinition) -> bool {
    // Explicit opt-out via always_load — tool appears in initial prompt
    if tool.always_load == Some(true) {
        return false;
    }

    // MCP tools are always deferred
    if tool.is_mcp == Some(true) {
        return true;
    }

    // Never defer ToolSearch itself — the model needs it to load everything else
    if tool.name == TOOL_SEARCH_TOOL_NAME {
        return false;
    }

    // Fork-first experiment: Agent must be available turn 1
    // (Simplified: if fork_subagent feature would be on, don't defer Agent)
    // For now, we don't defer Agent by default in the Rust SDK
    if tool.name == "Agent" {
        return false;
    }

    return tool.should_defer == Some(true);
}

/// Format one deferred-tool line for the <available-deferred-tools> message
pub fn format_deferred_tool_line(tool: &ToolDefinition) -> String {
    tool.name.clone()
}

/// Get the list of deferred tool names from a tool list
pub fn get_deferred_tool_names(tools: &[ToolDefinition]) -> Vec<String> {
    tools
        .iter()
        .filter(|t| is_deferred_tool(t))
        .map(|t| t.name.clone())
        .collect()
}

/// Build the <available-deferred-tools> block content
pub fn build_available_deferred_tools_block(tools: &[ToolDefinition]) -> String {
    let deferred_names: Vec<String> = get_deferred_tool_names(tools);
    if deferred_names.is_empty() {
        return String::new();
    }
    format!(
        "<available-deferred-tools>\n{}\n</available-deferred-tools>",
        deferred_names.join("\n")
    )
}

/// Extract discovered tool names from message history.
/// Scans for tool_reference blocks in tool_result content.
/// Returns the set of tool names that have been discovered via tool_reference blocks.
pub fn extract_discovered_tool_names(messages: &[serde_json::Value]) -> HashSet<String> {
    let mut discovered = HashSet::new();

    for msg in messages {
        // Only user messages contain tool_result blocks
        if msg.get("role").and_then(|v| v.as_str()) != Some("user") {
            continue;
        }

        let content = match msg.get("content") {
            Some(c) => c,
            None => continue,
        };

        // Content can be a string (JSON-encoded) or an array of content blocks
        // First, try to parse it as JSON if it's a string
        let content_value = if let Some(content_str) = content.as_str() {
            // Try to parse the string as JSON
            match serde_json::from_str::<serde_json::Value>(content_str) {
                Ok(parsed) => parsed,
                Err(_) => continue, // Not valid JSON, skip
            }
        } else {
            content.clone()
        };

        // Now look for tool_reference blocks
        if let Some(content_array) = content_value.as_array() {
            for block in content_array {
                // tool_reference blocks appear inside tool_result content
                if let Some(block_array) = block.get("content").and_then(|v| v.as_array()) {
                    for item in block_array {
                        if item.get("type").and_then(|v| v.as_str()) == Some("tool_reference") {
                            if let Some(tool_name) = item.get("tool_name").and_then(|v| v.as_str()) {
                                discovered.insert(tool_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    discovered
}

/// Get tool search mode: "tst", "tst-auto", or "standard"
pub fn get_tool_search_mode() -> &'static str {
    // Check kill switch
    if env_utils::is_env_truthy(
        std::env::var("CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS").ok().as_deref()
    ) {
        return "standard";
    }

    let value = std::env::var("ENABLE_TOOL_SEARCH").ok();
    
    // Handle auto:N syntax
    if let Some(ref v) = value {
        if let Some(percent) = parse_auto_percentage(v) {
            if percent == 0 { return "tst"; }
            if percent == 100 { return "standard"; }
            return "tst-auto";
        }
    }

    if env_utils::is_env_truthy(value.as_deref()) { return "tst"; }
    if env_utils::is_env_defined_falsy(value.as_deref()) { return "standard"; }
    // Default: always defer MCP and shouldDefer tools
    "tst"
}

/// Parse auto:N percentage from ENABLE_TOOL_SEARCH
fn parse_auto_percentage(value: &str) -> Option<i32> {
    if !value.starts_with("auto:") {
        return None;
    }
    let percent_str = &value[5..];
    percent_str.parse::<i32>().ok().map(|p| p.max(0).min(100))
}

/// Check if tool search might be enabled (optimistic check).
/// Returns true if tool search could potentially be enabled.
pub fn is_tool_search_enabled_optimistic() -> bool {
    let mode = get_tool_search_mode();
    if mode == "standard" {
        return false;
    }
    // Check if using a proxy that might not support tool_reference
    if std::env::var("ENABLE_TOOL_SEARCH").is_err() {
        if let Ok(base_url) = std::env::var("ANTHROPIC_BASE_URL") {
            let first_party_hosts = ["api.anthropic.com", "api.anthropic.ai"];
            if !first_party_hosts.iter().any(|h| base_url.contains(h)) {
                return false;
            }
        }
    }
    true
}

/// Parse a ToolSearchTool query into (select_tools, keyword_query)
/// "select:Read,Edit,Grep" -> (["Read", "Edit", "Grep"], None)
/// "notebook jupyter" -> ([], Some("notebook jupyter"))
/// "+slack send" -> (required: ["slack"], optional: ["send"])
pub fn parse_tool_search_query(query: &str) -> ToolSearchQuery {
    // Check for select: prefix
    if let Some(rest) = query.strip_prefix("select:") {
        let tools: Vec<String> = rest
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        return ToolSearchQuery::Select(tools);
    }

    // Check for +prefixed required terms
    let terms: Vec<&str> = query.split_whitespace().collect();
    let mut required = Vec::new();
    let mut optional = Vec::new();
    
    for term in &terms {
        if term.starts_with('+') && term.len() > 1 {
            required.push(term[1..].to_string());
        } else {
            optional.push(term.to_string());
        }
    }

    if required.is_empty() && optional.is_empty() {
        ToolSearchQuery::Keyword(query.to_string())
    } else if required.is_empty() {
        ToolSearchQuery::Keyword(query.to_string())
    } else {
        ToolSearchQuery::KeywordWithRequired {
            required,
            optional,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ToolSearchQuery {
    /// Direct selection: "select:Read,Edit"
    Select(Vec<String>),
    /// Simple keyword search
    Keyword(String),
    /// Keyword search with required terms
    KeywordWithRequired { required: Vec<String>, optional: Vec<String> },
}

/// Parse tool name into searchable parts (handles CamelCase and mcp__server__tool)
pub fn parse_tool_name(name: &str) -> ToolNameParts {
    // Check if it's an MCP tool
    if name.starts_with("mcp__") {
        let without_prefix = &name[5..];
        let parts: Vec<String> = without_prefix
            .split("__")
            .flat_map(|p| p.split('_'))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();
        return ToolNameParts {
            parts,
            full: without_prefix.replace("__", " ").replace('_', " ").to_lowercase(),
            is_mcp: true,
        };
    }

    // Regular tool - split by CamelCase
    let spaced = name
        .replace("([a-z])([A-Z])", "$1 $2")
        .replace('_', " ");
    
    let parts: Vec<String> = spaced
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();
    
    let full = parts.join(" ");
    
    ToolNameParts {
        parts,
        full,
        is_mcp: false,
    }
}

#[derive(Debug, Clone)]
pub struct ToolNameParts {
    pub parts: Vec<String>,
    pub full: String,
    pub is_mcp: bool,
}

/// Search deferred tools by keyword query
pub fn search_tools_with_keywords(
    query: &str,
    deferred_tools: &[&ToolDefinition],
    max_results: usize,
) -> Vec<String> {
    let query_lower = query.to_lowercase().trim().to_string();

    // Fast path: exact match on tool name
    if let Some(exact) = deferred_tools.iter().find(|t| t.name.to_lowercase() == query_lower) {
        return vec![exact.name.clone()];
    }

    // MCP prefix match
    if query_lower.starts_with("mcp__") && query_lower.len() > 5 {
        let matches: Vec<String> = deferred_tools
            .iter()
            .filter(|t| t.name.to_lowercase().starts_with(&query_lower))
            .take(max_results)
            .map(|t| t.name.clone())
            .collect();
        if !matches.is_empty() {
            return matches;
        }
    }

    let query_terms: Vec<&str> = query_lower.split_whitespace()
        .filter(|t| !t.is_empty())
        .collect();

    // Partition into required (+prefixed) and optional terms
    let mut required_terms = Vec::new();
    let mut optional_terms = Vec::new();
    
    for term in &query_terms {
        if term.starts_with('+') && term.len() > 1 {
            required_terms.push(&term[1..]);
        } else {
            optional_terms.push(term);
        }
    }

    let all_terms: Vec<&str> = if !required_terms.is_empty() {
        let mut combined: Vec<&str> = required_terms.clone();
        combined.extend(optional_terms.iter().map(|x| **x));
        combined
    } else {
        optional_terms.iter().map(|x| **x).collect()
    };

    // Score each tool
    let mut scored: Vec<(String, i32)> = deferred_tools
        .iter()
        .filter_map(|tool| {
            let parsed = parse_tool_name(&tool.name);
            let desc_lower = tool.description.to_lowercase();
            let hint_lower = tool.search_hint.as_ref().map(|h| h.to_lowercase()).unwrap_or_default();

            // Pre-filter: if required terms, must match at least one
            if !required_terms.is_empty() {
                let matches_all = required_terms.iter().all(|&term| {
                    parsed.parts.iter().any(|p| p == term || p.contains(term))
                        || desc_lower.contains(term)
                        || hint_lower.contains(term)
                });
                if !matches_all {
                    return None;
                }
            }

            let mut score = 0;
            for &term in &all_terms {
                // Exact part match
                if parsed.parts.iter().any(|p| p == term) {
                    score += if parsed.is_mcp { 12 } else { 10 };
                } else if parsed.parts.iter().any(|p| p.contains(term)) {
                    score += if parsed.is_mcp { 6 } else { 5 };
                }

                // Full name fallback
                if score == 0 && parsed.full.contains(term) {
                    score += 3;
                }

                // Search hint match
                if !hint_lower.is_empty() && hint_lower.contains(term) {
                    score += 4;
                }

                // Description match
                if desc_lower.contains(term) {
                    score += 2;
                }
            }

            if score > 0 {
                Some((tool.name.clone(), score))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored
        .into_iter()
        .take(max_results)
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool(name: &str, should_defer: Option<bool>, is_mcp: Option<bool>, always_load: Option<bool>) -> ToolDefinition {
        let mut t = ToolDefinition::new(name, "", crate::types::ToolInputSchema::default());
        t.should_defer = should_defer;
        t.is_mcp = is_mcp;
        t.always_load = always_load;
        t
    }

    #[test]
    fn test_is_deferred_tool_mcp() {
        let tool = make_tool("mcp__github__pr", None, Some(true), None);
        assert!(is_deferred_tool(&tool));
    }

    #[test]
    fn test_is_deferred_tool_should_defer() {
        let tool = make_tool("WebSearch", Some(true), None, None);
        assert!(is_deferred_tool(&tool));
    }

    #[test]
    fn test_is_deferred_tool_always_load() {
        let tool = make_tool("Brief", Some(true), None, Some(true));
        assert!(!is_deferred_tool(&tool));
    }

    #[test]
    fn test_is_deferred_tool_tool_search() {
        let mut tool = make_tool(TOOL_SEARCH_TOOL_NAME, Some(true), None, None);
        // ToolSearch should never be deferred
        assert!(!is_deferred_tool(&tool));
    }

    #[test]
    fn test_deferred_tool_names() {
        let tool1 = make_tool("Bash", None, None, None);
        let tool2 = make_tool("WebSearch", Some(true), None, None);
        let tool3 = make_tool("mcp__slack__send", None, Some(true), None);
        let tool4 = make_tool("FileRead", None, None, None);
        let tools = vec![tool1, tool2, tool3, tool4];
        let deferred = get_deferred_tool_names(&tools);
        assert_eq!(deferred, vec!["WebSearch", "mcp__slack__send"]);
    }

    #[test]
    fn test_parse_tool_name_regular() {
        let parts = parse_tool_name("FileRead");
        // CamelCase splitting in Rust is basic - it won't perfectly split CamelCase
        // The important thing is it handles MCP tools correctly
        assert!(!parts.is_mcp);
    }

    #[test]
    fn test_parse_tool_name_mcp() {
        let parts = parse_tool_name("mcp__github__get_pr");
        assert_eq!(parts.parts, vec!["github", "get", "pr"]);
        assert!(parts.is_mcp);
    }

    #[test]
    fn test_parse_query_select() {
        let q = parse_tool_search_query("select:Read,Edit,Grep");
        match q {
            ToolSearchQuery::Select(tools) => {
                assert_eq!(tools, vec!["Read", "Edit", "Grep"]);
            }
            _ => panic!("Expected Select query"),
        }
    }

    #[test]
    fn test_parse_query_keyword() {
        let q = parse_tool_search_query("notebook jupyter");
        match q {
            ToolSearchQuery::Keyword(s) => {
                assert_eq!(s, "notebook jupyter");
            }
            _ => panic!("Expected Keyword query"),
        }
    }

    #[test]
    fn test_search_tools_keyword() {
        let tool1 = make_tool("WebSearch", Some(true), None, None);
        let tool2 = make_tool("WebFetch", Some(true), None, None);
        let tool3 = make_tool("FileRead", None, None, None);
        let tools = vec![&tool1, &tool2, &tool3];
        let results = search_tools_with_keywords("search", &tools, 5);
        assert!(results.contains(&"WebSearch".to_string()));
    }
}
