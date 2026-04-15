use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    #[serde(default)]
    pub is_mcp: Option<bool>,
}

pub fn is_mcp_tool(tool: &Tool) -> bool {
    tool.is_mcp.as_ref().map(|v| *v).unwrap_or(false)
}

impl Tool {
    pub fn is_mcp_tool(&self) -> bool {
        self.is_mcp_tool.unwrap_or(false)
    }
}

const PR_ACTIVITY_TOOL_SUFFIXES: &[&str] = &["subscribe_pr_activity", "unsubscribe_pr_activity"];

pub fn is_pr_activity_subscription_tool(name: &str) -> bool {
    PR_ACTIVITY_TOOL_SUFFIXES
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

pub fn merge_and_filter_tools(
    initial_tools: Vec<Tool>,
    assembled: Vec<Tool>,
    mode: &str,
) -> Vec<Tool> {
    let mut combined = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for tool in initial_tools.into_iter().chain(assembled.into_iter()) {
        if !seen.contains(&tool.name) {
            seen.insert(tool.name.clone());
            combined.push(tool);
        }
    }

    let mut mcp_tools: Vec<Tool> = Vec::new();
    let mut builtin_tools: Vec<Tool> = Vec::new();

    for tool in combined {
        if tool.is_mcp_tool() {
            mcp_tools.push(tool);
        } else {
            builtin_tools.push(tool);
        }
    }

    builtin_tools.sort_by(|a, b| a.name.cmp(&b.name));
    mcp_tools.sort_by(|a, b| a.name.cmp(&b.name));

    let mut result = builtin_tools;
    result.extend(mcp_tools);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_activity_tool() {
        assert!(is_pr_activity_subscription_tool(
            "mcp_subscribe_pr_activity"
        ));
        assert!(!is_pr_activity_subscription_tool("other_tool"));
    }

    #[test]
    fn test_merge_tools() {
        let initial = vec![Tool {
            name: "tool1".to_string(),
            description: None,
            input_schema: serde_json::json!({}),
            is_mcp: Some(false),
        }];
        let assembled = vec![
            Tool {
                name: "tool1".to_string(),
                description: None,
                input_schema: serde_json::json!({}),
                is_mcp: Some(false),
            },
            Tool {
                name: "tool2".to_string(),
                description: None,
                input_schema: serde_json::json!({}),
                is_mcp: Some(true),
            },
        ];

        let result = merge_and_filter_tools(initial, assembled, "full");
        assert_eq!(result.len(), 2);
    }
}
