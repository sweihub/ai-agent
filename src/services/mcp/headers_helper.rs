// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/headersHelper.ts
//! Dynamic headers handling for MCP servers using headersHelper script

use std::collections::HashMap;

use crate::services::analytics::log_event;
use crate::services::mcp::types::*;
use crate::utils::config::check_has_trust_dialog_accepted;
use crate::utils::cwd::get_cwd;
use crate::utils::debug::log_ant_error;
use crate::utils::exec_file_no_throw::exec_file_no_throw_with_cwd;

/// Feedback channel for ANT users reporting issues
const FEEDBACK_CHANNEL: &str = "#briarpatch-cc";

/// Check if running in non-interactive mode (SDK: checks AI_CODE_NON_INTERACTIVE env var)
/// CLI: uses bootstrap state to track interactive vs CI/CD mode
fn is_non_interactive_session() -> bool {
    std::env::var("AI_CODE_NON_INTERACTIVE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Parse JSON string, returning None on parse failure (matches TS jsonParse behavior)
fn json_parse(s: &str) -> Option<serde_json::Value> {
    serde_json::from_str(s).ok()
}

/// Get dynamic headers for an MCP server using the headersHelper script
/// @param server_name The name of the MCP server
/// @param config The MCP server configuration
/// @returns Headers object or None if not configured or failed
pub async fn get_mcp_headers_from_helper(
    server_name: &str,
    config: &McpSseServerConfig,
) -> Option<HashMap<String, String>> {
    let headers_helper = config.headers_helper.as_ref()?;

    // Security check for project/local settings
    // Skip trust check in non-interactive mode (e.g., CI/CD, automation)
    if !is_non_interactive_session() {
        // Note: McpSseServerConfig doesn't expose scope directly.
        // For project/local MCP servers, trust should be confirmed before headersHelper runs.
        // SDK users can set AI_CODE_NON_INTERACTIVE=1 to skip this check.
        if !check_has_trust_dialog_accepted() {
            let error_msg = format!(
                "Security: headersHelper for MCP server '{}' executed before workspace trust is confirmed. \
                 If you see this message, post in {}.",
                server_name, FEEDBACK_CHANNEL
            );
            log_ant_error("MCP headersHelper invoked before trust check", &error_msg);
            log_event("tengu_mcp_headersHelper_missing_trust", Default::default());
            return None;
        }
    }

    let cwd = get_cwd();

    // Execute headersHelper script
    // Pass server context so one helper script can serve multiple MCP servers
    // (git credential-helper style). See deshaw/anthropic-issues#28.
    let exec_result = exec_file_no_throw_with_cwd(headers_helper, vec![], &cwd).await;

    if exec_result.code != 0 || exec_result.stdout.is_empty() {
        log::warn!(
            "headersHelper for MCP server '{}' did not return a valid value (code: {})",
            server_name,
            exec_result.code
        );
        return None;
    }

    let result = exec_result.stdout.trim();

    // Parse JSON
    let headers = match json_parse(result) {
        Some(serde_json::Value::Object(map)) => map,
        _ => {
            log::warn!(
                "headersHelper for MCP server '{}' must return a JSON object",
                server_name
            );
            return None;
        }
    };

    // Validate all values are strings
    for (key, value) in &headers {
        if !value.is_string() {
            log::warn!(
                "headersHelper for MCP server '{}' returned non-string value for key \"{}\": {}",
                server_name,
                key,
                value
            );
            return None;
        }
    }

    // Convert to HashMap<String, String>
    let headers: HashMap<String, String> = headers
        .into_iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
        .collect();

    log::debug!(
        "Successfully retrieved {} headers from headersHelper for server '{}'",
        headers.len(),
        server_name
    );

    Some(headers)
}

/// Get combined headers for an MCP server (static + dynamic)
/// @param server_name The name of the MCP server
/// @param config The MCP server configuration
/// @returns Combined headers object (dynamic headers override static headers)
pub async fn get_mcp_server_headers(
    server_name: &str,
    config: &McpSseServerConfig,
) -> HashMap<String, String> {
    let mut headers = config.headers.clone().unwrap_or_default();

    if let Some(dynamic_headers) = get_mcp_headers_from_helper(server_name, config).await {
        headers.extend(dynamic_headers);
    }

    headers
}

/// Get combined headers for HTTP server config
/// @param server_name The name of the MCP server
/// @param config The MCP HTTP server configuration
/// @returns Combined headers object
pub async fn get_mcp_http_server_headers(
    server_name: &str,
    config: &McpHttpServerConfig,
) -> HashMap<String, String> {
    let mut headers = config.headers.clone().unwrap_or_default();

    if let Some(headers_helper) = &config.headers_helper {
        // Security check - skip in non-interactive mode
        if !is_non_interactive_session() && !check_has_trust_dialog_accepted() {
            log::debug!(
                "Skipping headersHelper for MCP server '{}' due to untrusted workspace",
                server_name
            );
        } else {
            let cwd = get_cwd();
            let exec_result = exec_file_no_throw_with_cwd(headers_helper, vec![], &cwd).await;

            if exec_result.code == 0 && !exec_result.stdout.is_empty() {
                let result = exec_result.stdout.trim();
                if let Some(serde_json::Value::Object(map)) = json_parse(result) {
                    // Validate all values are strings
                    let valid = map.values().all(|v| v.is_string());
                    if valid {
                        let dynamic: HashMap<String, String> = map
                            .into_iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                            .collect();
                        headers.extend(dynamic);
                    }
                }
            }
        }
    }

    headers
}

/// Get combined headers for WebSocket server config
/// @param server_name The name of the MCP server
/// @param config The MCP WebSocket server configuration
/// @returns Combined headers object
pub async fn get_mcp_ws_server_headers(
    server_name: &str,
    config: &McpWebSocketServerConfig,
) -> HashMap<String, String> {
    let mut headers = config.headers.clone().unwrap_or_default();

    if let Some(headers_helper) = &config.headers_helper {
        // Security check - skip in non-interactive mode
        if !is_non_interactive_session() && !check_has_trust_dialog_accepted() {
            log::debug!(
                "Skipping headersHelper for MCP server '{}' due to untrusted workspace",
                server_name
            );
        } else {
            let cwd = get_cwd();
            let exec_result = exec_file_no_throw_with_cwd(headers_helper, vec![], &cwd).await;

            if exec_result.code == 0 && !exec_result.stdout.is_empty() {
                let result = exec_result.stdout.trim();
                if let Some(serde_json::Value::Object(map)) = json_parse(result) {
                    // Validate all values are strings
                    let valid = map.values().all(|v| v.is_string());
                    if valid {
                        let dynamic: HashMap<String, String> = map
                            .into_iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                            .collect();
                        headers.extend(dynamic);
                    }
                }
            }
        }
    }

    headers
}
