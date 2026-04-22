// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/config.ts
//! MCP configuration management

use std::collections::HashMap;
use std::path::PathBuf;

use crate::services::mcp::normalization::normalize_name_for_mcp;
use crate::services::mcp::types::*;

/// CCR proxy URL path markers for remote sessions
const CCR_PROXY_PATH_MARKERS: &[&str] = &["/v2/session_ingress/shttp/mcp/", "/v2/ccr-sessions/"];

/// Extract command array from server config (stdio servers only)
/// Returns None for non-stdio servers
pub fn get_server_command_array(config: &McpServerConfig) -> Option<Vec<String>> {
    match config {
        McpServerConfig::Stdio(stdio) => {
            let mut cmd = vec![stdio.command.clone()];
            cmd.extend(stdio.args.clone());
            Some(cmd)
        }
        _ => None,
    }
}

/// Check if two command arrays match exactly
fn command_arrays_match(a: &[String], b: &[String]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| x == y)
}

/// Extract URL from server config (remote servers only)
/// Returns None for stdio/sdk servers
pub fn get_server_url(config: &McpServerConfig) -> Option<String> {
    match config {
        McpServerConfig::Sse(sse) => Some(sse.url.clone()),
        McpServerConfig::SseIde(sse_ide) => Some(sse_ide.url.clone()),
        McpServerConfig::WebSocketIde(ws_ide) => Some(ws_ide.url.clone()),
        McpServerConfig::Http(http) => Some(http.url.clone()),
        McpServerConfig::WebSocket(ws) => Some(ws.url.clone()),
        _ => None,
    }
}

/// If the URL is a CCR proxy URL, extract the original vendor URL from the
/// mcp_url query parameter. Otherwise return the URL unchanged.
pub fn unwrap_ccr_proxy_url(url: &str) -> String {
    if !CCR_PROXY_PATH_MARKERS.iter().any(|m| url.contains(m)) {
        return url.to_string();
    }

    // Try to extract mcp_url query param
    if let Some(idx) = url.find('?') {
        let path = &url[..idx];
        let query = &url[idx + 1..];
        if query.contains("mcp_url=") {
            for param in query.split('&') {
                if param.starts_with("mcp_url=") {
                    if let Ok(decoded) = urlencoding_decode(&param[8..]) {
                        return decoded;
                    }
                }
            }
        }
    }
    url.to_string()
}

/// Simple URL decode (percent encoding)
fn urlencoding_decode(input: &str) -> Result<String, ()> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            return Err(());
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

/// Compute a dedup signature for an MCP server config.
/// Two configs with the same signature are considered "the same server" for
/// plugin deduplication. Ignores env and headers.
pub fn get_mcp_server_signature(config: &McpServerConfig) -> Option<String> {
    if let Some(cmd) = get_server_command_array(config) {
        let json = serde_json::to_string(&cmd).unwrap_or_default();
        return Some(format!("stdio:{}", json));
    }

    if let Some(url) = get_server_url(config) {
        return Some(format!("url:{}", unwrap_ccr_proxy_url(&url)));
    }

    None
}

/// Filter plugin MCP servers, dropping any whose signature matches a
/// manually-configured server or an earlier-loaded plugin server.
pub fn dedup_plugin_mcp_servers(
    plugin_servers: &HashMap<String, ScopedMcpServerConfig>,
    manual_servers: &HashMap<String, ScopedMcpServerConfig>,
) -> (
    HashMap<String, ScopedMcpServerConfig>,
    Vec<SuppressedServer>,
) {
    // Map signature -> server name
    let mut manual_sigs: HashMap<String, String> = HashMap::new();
    for (name, config) in manual_servers {
        if let Some(sig) = get_mcp_server_signature(&config.config) {
            manual_sigs.entry(sig).or_insert_with(|| name.clone());
        }
    }

    let mut servers: HashMap<String, ScopedMcpServerConfig> = HashMap::new();
    let mut suppressed: Vec<SuppressedServer> = Vec::new();
    let mut seen_plugin_sigs: HashMap<String, String> = HashMap::new();

    for (name, config) in plugin_servers {
        let sig = match get_mcp_server_signature(&config.config) {
            Some(s) => s,
            None => {
                servers.insert(name.clone(), config.clone());
                continue;
            }
        };

        if let Some(manual_dup) = manual_sigs.get(&sig) {
            log::debug!(
                "Suppressing plugin MCP server \"{}\": duplicates manually-configured \"{}\"",
                name,
                manual_dup
            );
            suppressed.push(SuppressedServer {
                name: name.clone(),
                duplicate_of: manual_dup.clone(),
            });
            continue;
        }

        if let Some(plugin_dup) = seen_plugin_sigs.get(&sig) {
            log::debug!(
                "Suppressing plugin MCP server \"{}\": duplicates earlier plugin server \"{}\"",
                name,
                plugin_dup
            );
            suppressed.push(SuppressedServer {
                name: name.clone(),
                duplicate_of: plugin_dup.clone(),
            });
            continue;
        }

        seen_plugin_sigs.insert(sig, name.clone());
        servers.insert(name.clone(), config.clone());
    }

    (servers, suppressed)
}

/// Suppressed server info
#[derive(Debug, Clone)]
pub struct SuppressedServer {
    pub name: String,
    pub duplicate_of: String,
}

/// Convert a URL pattern with wildcards to a regex pattern
fn url_pattern_to_regex(pattern: &str) -> String {
    // Escape regex special characters except *
    let escaped: String = pattern
        .chars()
        .map(|c| {
            if "+?^${()|[]\\".contains(c) {
                format!("\\{}", c)
            } else {
                c.to_string()
            }
        })
        .collect();
    // Replace * with regex equivalent
    escaped.replace('*', ".*")
}

/// Check if a URL matches a pattern with wildcard support
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    let regex_str = format!("^{}$", url_pattern_to_regex(pattern));
    regex::Regex::new(&regex_str)
        .map(|re| re.is_match(url))
        .unwrap_or(false)
}

/// Check if an MCP server is disabled
pub fn is_mcp_server_disabled(server_name: &str, disabled_servers: Option<&[String]>) -> bool {
    if let Some(disabled) = disabled_servers {
        let normalized = normalize_name_for_mcp(server_name);
        disabled
            .iter()
            .any(|name| normalize_name_for_mcp(name) == normalized)
    } else {
        false
    }
}

/// Check if an MCP server is denied by enterprise policy
/// Checks name-based, command-based, and URL-based restrictions
pub fn is_mcp_server_denied(
    server_name: &str,
    config: Option<&McpServerConfig>,
    denied_servers: Option<&[McpServerDenialEntry]>,
) -> bool {
    let Some(denied) = denied_servers else {
        return false;
    };

    // Check name-based denial
    for entry in denied {
        match entry {
            McpServerDenialEntry::Name(name) => {
                if name == server_name {
                    return true;
                }
            }
            McpServerDenialEntry::Command(cmd) => {
                if let Some(cfg) = config {
                    if let Some(server_cmd) = get_server_command_array(cfg) {
                        if command_arrays_match(&server_cmd, cmd) {
                            return true;
                        }
                    }
                }
            }
            McpServerDenialEntry::Url(url_pattern) => {
                if let Some(cfg) = config {
                    if let Some(server_url) = get_server_url(cfg) {
                        if url_matches_pattern(&server_url, url_pattern) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Check if an MCP server is allowed by enterprise policy
pub fn is_mcp_server_allowed_by_policy(
    server_name: &str,
    config: Option<&McpServerConfig>,
    allowed_servers: Option<&[McpServerAllowanceEntry]>,
    denied_servers: Option<&[McpServerDenialEntry]>,
) -> bool {
    // Denylist takes absolute precedence
    if is_mcp_server_denied(server_name, config, denied_servers) {
        return false;
    }

    let Some(allowed) = allowed_servers else {
        return true; // No allowlist restrictions
    };

    // Empty allowlist means block all servers
    if allowed.is_empty() {
        return false;
    }

    // Check if allowlist contains command-based or URL-based entries
    let has_command_entries = allowed
        .iter()
        .any(|e| matches!(e, McpServerAllowanceEntry::Command(_)));
    let has_url_entries = allowed
        .iter()
        .any(|e| matches!(e, McpServerAllowanceEntry::Url(_)));

    if let Some(cfg) = config {
        if let Some(server_cmd) = get_server_command_array(cfg) {
            // This is a stdio server
            if has_command_entries {
                for entry in allowed {
                    if let McpServerAllowanceEntry::Command(cmd) = entry {
                        if command_arrays_match(&server_cmd, cmd) {
                            return true;
                        }
                    }
                }
                return false;
            } else {
                // No command entries, check name-based allowance
                for entry in allowed {
                    if let McpServerAllowanceEntry::Name(name) = entry {
                        if name == server_name {
                            return true;
                        }
                    }
                }
                return false;
            }
        } else if let Some(server_url) = get_server_url(cfg) {
            // This is a remote server
            if has_url_entries {
                for entry in allowed {
                    if let McpServerAllowanceEntry::Url(pattern) = entry {
                        if url_matches_pattern(&server_url, pattern) {
                            return true;
                        }
                    }
                }
                return false;
            } else {
                // No URL entries, check name-based allowance
                for entry in allowed {
                    if let McpServerAllowanceEntry::Name(name) = entry {
                        if name == server_name {
                            return true;
                        }
                    }
                }
                return false;
            }
        }
    }

    // No config provided, check name-based allowance only
    for entry in allowed {
        if let McpServerAllowanceEntry::Name(name) = entry {
            if name == server_name {
                return true;
            }
        }
    }
    false
}

/// MCP server denial entry
#[derive(Debug, Clone)]
pub enum McpServerDenialEntry {
    Name(String),
    Command(Vec<String>),
    Url(String),
}

/// MCP server allowance entry
#[derive(Debug, Clone)]
pub enum McpServerAllowanceEntry {
    Name(String),
    Command(Vec<String>),
    Url(String),
}

/// Add scope to server configs
pub fn add_scope_to_servers(
    servers: &HashMap<String, McpServerConfig>,
    scope: ConfigScope,
) -> HashMap<String, ScopedMcpServerConfig> {
    servers
        .iter()
        .map(|(name, config)| {
            (
                name.clone(),
                ScopedMcpServerConfig {
                    config: config.clone(),
                    scope: scope.clone(),
                    plugin_source: None,
                },
            )
        })
        .collect()
}

/// Get MCP config file path for project
pub fn get_project_mcp_file_path(cwd: &PathBuf) -> PathBuf {
    cwd.join(".mcp.json")
}

/// Get global MCP config file path
pub fn get_global_mcp_file_path() -> PathBuf {
    // TODO: Get from settings
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ai-agent")
        .join("mcp.json")
}

/// Get enterprise MCP config file path
pub fn get_enterprise_mcp_file_path() -> PathBuf {
    // TODO: Get from managed settings
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ai-agent")
        .join("managed-mcp.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_server_command_array_stdio() {
        let config = McpServerConfig::Stdio(McpStdioServerConfig {
            config_type: Some("stdio".to_string()),
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: None,
        });

        let cmd = get_server_command_array(&config);
        assert_eq!(cmd, Some(vec!["node".to_string(), "server.js".to_string()]));
    }

    #[test]
    fn test_get_server_command_array_non_stdio() {
        let config = McpServerConfig::Http(McpHttpServerConfig {
            config_type: "http".to_string(),
            url: "https://example.com".to_string(),
            headers: None,
            headers_helper: None,
            oauth: None,
        });

        let cmd = get_server_command_array(&config);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_get_server_url_http() {
        let config = McpServerConfig::Http(McpHttpServerConfig {
            config_type: "http".to_string(),
            url: "https://example.com/mcp".to_string(),
            headers: None,
            headers_helper: None,
            oauth: None,
        });

        let url = get_server_url(&config);
        assert_eq!(url, Some("https://example.com/mcp".to_string()));
    }

    #[test]
    fn test_url_matches_pattern() {
        assert!(url_matches_pattern(
            "https://example.com/api/v1",
            "https://example.com/*"
        ));
        assert!(url_matches_pattern(
            "https://api.example.com/path",
            "https://*.example.com/*"
        ));
        assert!(!url_matches_pattern(
            "https://other.com/path",
            "https://example.com/*"
        ));
    }

    #[test]
    fn test_mcp_server_signature() {
        let config = McpServerConfig::Stdio(McpStdioServerConfig {
            config_type: Some("stdio".to_string()),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "server".to_string()],
            env: None,
        });

        let sig = get_mcp_server_signature(&config);
        assert!(sig.is_some());
        assert!(sig.unwrap().starts_with("stdio:"));
    }

    #[test]
    fn test_command_arrays_match() {
        assert!(command_arrays_match(
            &["node".to_string(), "server.js".to_string()],
            &["node".to_string(), "server.js".to_string()]
        ));
        assert!(!command_arrays_match(
            &["node".to_string(), "server.js".to_string()],
            &["node".to_string()]
        ));
    }
}
