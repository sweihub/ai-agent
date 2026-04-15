// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/utils.ts
//! MCP utility functions

use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::services::mcp::normalization::normalize_name_for_mcp;
use crate::services::mcp::types::*;

/// Tool reference for MCP utilities (minimal version for filtering)
#[derive(Debug, Clone, Default)]
pub struct McpToolRef {
    pub name: Option<String>,
    pub is_mcp: Option<bool>,
}

/// Command reference for MCP utilities (minimal version for filtering)
#[derive(Debug, Clone, Default)]
pub struct McpCommandRef {
    pub name: Option<String>,
    pub command_type: Option<String>,
    pub loaded_from: Option<String>,
    pub is_mcp: Option<bool>,
}

/// Filters tools by MCP server name
pub fn filter_tools_by_server(tools: &[McpToolRef], server_name: &str) -> Vec<McpToolRef> {
    let prefix = format!("mcp__{}_", normalize_name_for_mcp(server_name));
    tools
        .iter()
        .filter(|tool| tool.name.as_ref().map_or(false, |name| name.starts_with(&prefix)))
        .cloned()
        .collect()
}

/// True when a command belongs to the given MCP server.
/// MCP prompts are named `mcp__<server>__<prompt>` (wire-format constraint);
/// MCP skills are named `<server>:<skill>` (matching plugin/nested-dir skill naming).
pub fn command_belongs_to_server(command: &McpCommandRef, server_name: &str) -> bool {
    let normalized = normalize_name_for_mcp(server_name);
    command.name.as_ref().map_or(false, |name| {
        name.starts_with(&format!("mcp__{}_", normalized)) || name.starts_with(&format!("{}:", normalized))
    })
}

/// Filters commands by MCP server name
pub fn filter_commands_by_server(commands: &[McpCommandRef], server_name: &str) -> Vec<McpCommandRef> {
    commands
        .iter()
        .filter(|c| command_belongs_to_server(c, server_name))
        .cloned()
        .collect()
}

/// Filters MCP prompts (not skills) by server
pub fn filter_mcp_prompts_by_server(commands: &[McpCommandRef], server_name: &str) -> Vec<McpCommandRef> {
    commands
        .iter()
        .filter(|c| {
            command_belongs_to_server(c, server_name)
                && !(c.command_type.as_deref() == Some("prompt") && c.loaded_from.as_deref() == Some("mcp"))
        })
        .cloned()
        .collect()
}

/// Filters resources by MCP server name
pub fn filter_resources_by_server(resources: &[ServerResource], server_name: &str) -> Vec<ServerResource> {
    resources
        .iter()
        .filter(|resource| resource.server == server_name)
        .cloned()
        .collect()
}

/// Removes tools belonging to a specific MCP server
pub fn exclude_tools_by_server(tools: &[McpToolRef], server_name: &str) -> Vec<McpToolRef> {
    let prefix = format!("mcp__{}_", normalize_name_for_mcp(server_name));
    tools
        .iter()
        .filter(|tool| tool.name.as_ref().map_or(true, |name| !name.starts_with(&prefix)))
        .cloned()
        .collect()
}

/// Removes commands belonging to a specific MCP server
pub fn exclude_commands_by_server(commands: &[McpCommandRef], server_name: &str) -> Vec<McpCommandRef> {
    commands
        .iter()
        .filter(|c| !command_belongs_to_server(c, server_name))
        .cloned()
        .collect()
}

/// Removes resources belonging to a specific MCP server
pub fn exclude_resources_by_server(resources: &HashMap<String, Vec<ServerResource>>, server_name: &str) -> HashMap<String, Vec<ServerResource>> {
    resources
        .iter()
        .filter(|(name, _)| *name != server_name)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Stable hash of an MCP server config for change detection.
/// Excludes `scope` (provenance, not content).
/// Keys are sorted for stability so {a:1,b:2} and {b:2,a:1} hash the same.
pub fn hash_mcp_config(config: &ScopedMcpServerConfig) -> String {
    // Convert to JSON first, then sort keys recursively for stable serialization
    let json_value = serde_json::to_value(config).unwrap_or_default();
    let stable = sort_keys_for_stability(&json_value);
    let json = serde_json::to_string(&stable).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

/// Recursively sort object keys for stable JSON serialization
fn sort_keys_for_stability(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut sorted: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for k in keys {
                sorted.insert(k.clone(), sort_keys_for_stability(&map[k]));
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sort_keys_for_stability).collect())
        }
        _ => value.clone(),
    }
}

/// Parses header strings into a map
pub fn parse_headers(header_array: &[String]) -> Result<HashMap<String, String>, String> {
    let mut headers = HashMap::new();

    for header in header_array {
        let colon_index = header.find(':').ok_or_else(|| {
            format!("Invalid header format: \"{}\". Expected format: \"Header-Name: value\"", header)
        })?;

        let key = header[..colon_index].trim();
        let value = header[colon_index + 1..].trim();

        if key.is_empty() {
            return Err(format!("Invalid header: \"{}\". Header name cannot be empty.", header));
        }

        headers.insert(key.to_string(), value.to_string());
    }

    Ok(headers)
}

/// Get project MCP server status
pub fn get_project_mcp_server_status(
    server_name: &str,
    disabled_servers: Option<&[String]>,
    enabled_servers: Option<&[String]>,
    enable_all: bool,
) -> &'static str {
    let normalized_name = normalize_name_for_mcp(server_name);

    if let Some(disabled) = disabled_servers {
        if disabled.iter().any(|name| normalize_name_for_mcp(name) == normalized_name) {
            return "rejected";
        }
    }

    if let Some(enabled) = enabled_servers {
        if enabled.iter().any(|name| normalize_name_for_mcp(name) == normalized_name) || enable_all {
            return "approved";
        }
    }

    "pending"
}

/// Ensure config scope is valid
pub fn ensure_config_scope(scope: Option<&str>) -> ConfigScope {
    match scope {
        None => ConfigScope::Local,
        Some("local") => ConfigScope::Local,
        Some("user") => ConfigScope::User,
        Some("project") => ConfigScope::Project,
        Some("dynamic") => ConfigScope::Dynamic,
        Some("enterprise") => ConfigScope::Enterprise,
        Some("claudeai") => ConfigScope::ClaudeAi,
        Some("managed") => ConfigScope::Managed,
        Some(_) => ConfigScope::Local, // Default to local for unknown scopes
    }
}

/// Ensure transport type is valid
pub fn ensure_transport(transport_type: Option<&str>) -> &'static str {
    match transport_type {
        None => "stdio",
        Some("stdio") => "stdio",
        Some("sse") => "sse",
        Some("http") => "http",
        Some("ws") => "ws",
        Some("sdk") => "sdk",
        Some(_) => "stdio", // Default to stdio for unknown types
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_tools_by_server() {
        let tools = vec![
            McpToolRef { name: Some("mcp__server1__tool1".to_string()), is_mcp: None },
            McpToolRef { name: Some("mcp__server2__tool2".to_string()), is_mcp: None },
            McpToolRef { name: Some("other_tool".to_string()), is_mcp: None },
        ];

        let filtered = filter_tools_by_server(&tools, "server1");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, Some("mcp__server1__tool1".to_string()));
    }

    #[test]
    fn test_command_belongs_to_server() {
        let cmd1 = McpCommandRef { name: Some("mcp__server1__prompt".to_string()), command_type: None, loaded_from: None, is_mcp: None };
        let cmd2 = McpCommandRef { name: Some("server1:skill".to_string()), command_type: None, loaded_from: None, is_mcp: None };
        let cmd3 = McpCommandRef { name: Some("other_command".to_string()), command_type: None, loaded_from: None, is_mcp: None };

        assert!(command_belongs_to_server(&cmd1, "server1"));
        assert!(command_belongs_to_server(&cmd2, "server1"));
        assert!(!command_belongs_to_server(&cmd3, "server1"));
    }

    #[test]
    fn test_parse_headers() {
        let headers = vec!["Content-Type: application/json".to_string(), "Authorization: Bearer token".to_string()];
        let result = parse_headers(&headers).unwrap();
        assert_eq!(result.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(result.get("Authorization"), Some(&"Bearer token".to_string()));
    }

    #[test]
    fn test_parse_headers_invalid() {
        let headers = vec!["InvalidHeader".to_string()];
        let result = parse_headers(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure_config_scope() {
        assert_eq!(ensure_config_scope(None), ConfigScope::Local);
        assert_eq!(ensure_config_scope(Some("user")), ConfigScope::User);
        assert_eq!(ensure_config_scope(Some("invalid")), ConfigScope::Local);
    }

    #[test]
    fn test_ensure_transport() {
        assert_eq!(ensure_transport(None), "stdio");
        assert_eq!(ensure_transport(Some("sse")), "sse");
        assert_eq!(ensure_transport(Some("http")), "http");
        assert_eq!(ensure_transport(Some("invalid")), "stdio");
    }

    #[test]
    fn test_hash_mcp_config() {
        let config = ScopedMcpServerConfig {
            config: McpServerConfig::Stdio(McpStdioServerConfig {
                config_type: Some("stdio".to_string()),
                command: "node".to_string(),
                args: vec!["server.js".to_string()],
                env: None,
            }),
            scope: ConfigScope::Local,
            plugin_source: None,
        };

        let hash1 = hash_mcp_config(&config);
        let hash2 = hash_mcp_config(&config);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16);
    }
}