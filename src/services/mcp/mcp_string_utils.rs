use crate::services::mcp::normalization::normalize_name_for_mcp;

#[derive(Debug, Clone)]
pub struct McpInfo {
    pub server_name: String,
    pub tool_name: Option<String>,
}

pub fn mcp_info_from_string(tool_string: &str) -> Option<McpInfo> {
    let parts: Vec<&str> = tool_string.split("__").collect();
    if parts.len() < 3 || parts[0] != "mcp" || parts[1].is_empty() {
        return None;
    }

    let server_name = parts[1].to_string();
    let tool_name = if parts.len() > 2 {
        Some(parts[2..].join("__"))
    } else {
        None
    };

    Some(McpInfo {
        server_name,
        tool_name,
    })
}

pub fn get_mcp_prefix(server_name: &str) -> String {
    format!("mcp__{}_", normalize_name_for_mcp(server_name))
}

pub fn build_mcp_tool_name(server_name: &str, tool_name: &str) -> String {
    format!(
        "{}{}",
        get_mcp_prefix(server_name),
        normalize_name_for_mcp(tool_name)
    )
}

pub fn get_tool_name_for_permission_check(tool: &McpTool) -> String {
    match &tool.mcp_info {
        Some(info) => {
            build_mcp_tool_name(&info.server_name, info.tool_name.as_deref().unwrap_or(""))
        }
        None => tool.name.clone(),
    }
}

#[derive(Debug, Clone)]
pub struct McpTool {
    pub name: String,
    pub mcp_info: Option<McpInfo>,
}

pub fn get_mcp_display_name(full_name: &str, server_name: &str) -> String {
    let prefix = format!("mcp__{}_", normalize_name_for_mcp(server_name));
    full_name
        .strip_prefix(&prefix)
        .unwrap_or(full_name)
        .to_string()
}

/// Regex to strip (MCP) suffix with optional surrounding whitespace
static MCP_SUFFIX_RE: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"\s*\(MCP\)\s*$").unwrap());

pub fn extract_mcp_tool_display_name(user_facing_name: &str) -> String {
    // TypeScript: replace(/\s*\(MCP\)\s*$/, '').trim()
    let without_suffix = MCP_SUFFIX_RE
        .replace(user_facing_name, "")
        .trim()
        .to_string();

    if let Some(dash_index) = without_suffix.find(" - ") {
        without_suffix[dash_index + 3..].trim().to_string()
    } else {
        without_suffix.to_string()
    }
}
