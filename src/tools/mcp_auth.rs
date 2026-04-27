// Source: ~/claudecode/openclaudecode/src/tools/McpAuthTool/McpAuthTool.ts
use crate::error::AgentError;
use crate::services::mcp::auth::{McpOAuthResult, McpOAuthStatus, perform_mcp_oauth_flow};
use crate::services::mcp::client::clear_mcp_auth_cache;
use crate::types::*;

pub const MCP_AUTH_TOOL_NAME: &str = "mcp_authenticate";

pub const DESCRIPTION: &str =
    "Authenticate an MCP server that requires OAuth. Returns an authorization URL for the user to complete the flow.";

/// McpAuthTool - pseudo-tool for MCP servers that need authentication.
///
/// In TypeScript, createMcpAuthTool creates a pseudo-tool for an MCP server
/// that is installed but not authenticated. When called, it starts the OAuth
/// flow and returns the authorization URL.
///
/// The Rust version dispatches through `perform_mcp_oauth_flow()` which uses
/// a globally-registered callback. SDK users call `register_mcp_oauth_callback()`
/// to wire this up.
pub struct McpAuthTool;

impl McpAuthTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        MCP_AUTH_TOOL_NAME
    }

    pub fn description(&self) -> &str {
        DESCRIPTION
    }

    pub fn user_facing_name(&self, input: Option<&serde_json::Value>) -> String {
        if let Some(server) = input.and_then(|i| i["server"].as_str()) {
            format!("{} - authenticate (MCP)", server)
        } else {
            "Authenticate MCP Server".to_string()
        }
    }

    pub fn get_tool_use_summary(&self, input: Option<&serde_json::Value>) -> Option<String> {
        input.and_then(|inp| inp["server"].as_str().map(String::from))
    }

    pub fn render_tool_result_message(
        &self,
        content: &serde_json::Value,
    ) -> Option<String> {
        content["content"].as_str().map(|s| s.to_string())
    }

    pub fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({
                "server": {
                    "type": "string",
                    "description": "The MCP server name to authenticate"
                },
                "transport": {
                    "type": "string",
                    "description": "The transport type (sse, http, stdio, claudeai-proxy)"
                }
            }),
            required: Some(vec!["server".to_string()]),
        }
    }

    pub async fn execute(
        &self,
        input: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, AgentError> {
        let server = input["server"]
            .as_str()
            .ok_or_else(|| AgentError::Tool("Missing server parameter".to_string()))?;

        // Handle claude.ai proxy connectors — not programmatically authable
        if input["transport"].as_str() == Some("claudeai-proxy") {
            return Ok(ToolResult {
                result_type: "text".to_string(),
                tool_use_id: "".to_string(),
                content: format!(
                    "This is a claude.ai MCP connector. Ask the user to run /mcp and select \"{}\" to authenticate.",
                    server
                ),
                is_error: None,
                was_persisted: None,
            });
        }

        // OAuth flow only supports SSE and HTTP transports
        let transport = input["transport"].as_str().unwrap_or("stdio");
        if !(transport == "sse" || transport == "http") {
            return Ok(ToolResult {
                result_type: "text".to_string(),
                tool_use_id: "".to_string(),
                content: format!(
                    "Server \"{}\" uses {} transport which does not support OAuth from this tool. \
                     Ask the user to run /mcp and authenticate manually.",
                    server, transport
                ),
                is_error: None,
                was_persisted: None,
            });
        }

        // Build config JSON for the callback
        let config = input.get("config").cloned().unwrap_or(serde_json::json!({
            "type": transport,
            "url": input["url"].as_str().map(String::from),
        }));

        // Start the OAuth flow — the callback reports the auth URL via on_auth_url
        let server_name = server.to_string();
        let auth_url_tx: std::sync::Arc<parking_lot::Mutex<Option<String>>> = std::sync::Arc::new(parking_lot::Mutex::new(None));
        let auth_url_clone = auth_url_tx.clone();

        let on_auth_url: Option<std::sync::Arc<dyn Fn(String) + Send + Sync>> = Some(
            std::sync::Arc::new(move |url: String| {
                let mut guard = auth_url_clone.lock();
                *guard = Some(url);
            }) as std::sync::Arc<dyn Fn(String) + Send + Sync>,
        );

        match perform_mcp_oauth_flow(server_name.clone(), config.clone(), on_auth_url).await {
            Ok(McpOAuthResult { status, message, auth_url }) => {
                // Clear the auth cache so the server reconnects with fresh tokens
                clear_mcp_auth_cache();

                let url = auth_url.or_else(|| auth_url_tx.lock().take());

                if status == McpOAuthStatus::Authenticated {
                    return Ok(ToolResult {
                        result_type: "text".to_string(),
                        tool_use_id: "".to_string(),
                        content: format!(
                            "Authentication completed silently for {}. The server's tools should now be available.",
                            server
                        ),
                        is_error: None,
                        was_persisted: None,
                    });
                }

                if let Some(auth_url) = url {
                    return Ok(ToolResult {
                        result_type: "text".to_string(),
                        tool_use_id: "".to_string(),
                        content: format!(
                            "Ask the user to open this URL in their browser to authorize the {} MCP server:\n\n{}\n\nOnce they complete the flow, the server's tools will become available automatically.",
                            server, auth_url
                        ),
                        is_error: None,
                        was_persisted: None,
                    });
                }

                Ok(ToolResult {
                    result_type: "text".to_string(),
                    tool_use_id: "".to_string(),
                    content: message,
                    is_error: Some(status == McpOAuthStatus::Error),
                    was_persisted: None,
                })
            }
            Err(e) => Ok(ToolResult {
                result_type: "text".to_string(),
                tool_use_id: "".to_string(),
                content: format!(
                    "Failed to start OAuth flow for {}: {}. Ask the user to run /mcp and authenticate manually.",
                    server, e
                ),
                is_error: Some(true),
                was_persisted: None,
            }),
        }
    }
}

impl Default for McpAuthTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_auth_tool_name() {
        let tool = McpAuthTool::new();
        assert_eq!(tool.name(), MCP_AUTH_TOOL_NAME);
    }

    #[test]
    fn test_mcp_auth_tool_description() {
        let tool = McpAuthTool::new();
        assert_eq!(tool.description(), DESCRIPTION);
    }

    #[test]
    fn test_mcp_auth_tool_user_facing_name_with_server() {
        let tool = McpAuthTool::new();
        let name = tool.user_facing_name(Some(&serde_json::json!({"server": "myServer"})));
        assert_eq!(name, "myServer - authenticate (MCP)");
    }

    #[test]
    fn test_mcp_auth_tool_user_facing_name_without_server() {
        let tool = McpAuthTool::new();
        let name = tool.user_facing_name(None);
        assert_eq!(name, "Authenticate MCP Server");
    }

    #[test]
    fn test_mcp_auth_tool_input_schema_has_server() {
        let tool = McpAuthTool::new();
        let schema = tool.input_schema();
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties["server"].is_object());
        assert!(schema.required.as_ref().unwrap().contains(&"server".to_string()));
    }

    #[tokio::test]
    async fn test_mcp_auth_tool_missing_server() {
        let tool = McpAuthTool::new();
        let result = tool.execute(serde_json::json!({}), &ToolContext::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mcp_auth_tool_returns_unsupported_message() {
        let tool = McpAuthTool::new();
        let result = tool
            .execute(
                serde_json::json!({"server": "testServer", "transport": "stdio"}),
                &ToolContext::default(),
            )
            .await;
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.content.contains("stdio"));
        assert!(r.content.contains("testServer"));
    }

    #[tokio::test]
    async fn test_mcp_auth_tool_claudeai_unsupported() {
        let tool = McpAuthTool::new();
        let result = tool
            .execute(
                serde_json::json!({"server": "testServer", "transport": "claudeai-proxy"}),
                &ToolContext::default(),
            )
            .await;
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.content.contains("claude.ai"));
    }

    #[tokio::test]
    async fn test_mcp_auth_tool_sse_no_callback() {
        let tool = McpAuthTool::new();
        let result = tool
            .execute(
                serde_json::json!({"server": "testServer", "transport": "sse"}),
                &ToolContext::default(),
            )
            .await;
        // Returns error because no OAuth callback is registered
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.content.contains("Failed to start OAuth"));
    }
}
