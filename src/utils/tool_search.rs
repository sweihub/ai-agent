use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

pub fn search_tools(tools: &[ToolInfo], query: &str) -> Vec<ToolInfo> {
    let query_lower = query.to_lowercase();

    tools
        .iter()
        .filter(|tool| {
            tool.name.to_lowercase().contains(&query_lower)
                || tool.description.to_lowercase().contains(&query_lower)
                || tool
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect()
}

pub fn filter_tools_by_category(tools: &[ToolInfo], category: &str) -> Vec<ToolInfo> {
    tools
        .iter()
        .filter(|tool| tool.category.as_deref() == Some(category))
        .cloned()
        .collect()
}

pub fn get_tool_suggestions(tools: &[ToolInfo], partial: &str) -> Vec<String> {
    let partial_lower = partial.to_lowercase();

    tools
        .iter()
        .filter(|tool| tool.name.to_lowercase().starts_with(&partial_lower))
        .map(|t| t.name.clone())
        .take(5)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_tools() {
        let tools = vec![ToolInfo {
            name: "read".to_string(),
            description: "Read a file".to_string(),
            category: Some("file".to_string()),
            tags: vec!["io".to_string()],
        }];

        let results = search_tools(&tools, "read");
        assert_eq!(results.len(), 1);
    }
}
