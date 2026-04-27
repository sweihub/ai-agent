// Source: ~/claudecode/openclaudecode/src/tools/MCPTool/MCPTool.ts
use crate::error::AgentError;
use crate::types::*;

pub const MCP_TOOL_NAME: &str = "mcp";

pub const DESCRIPTION: &str =
    "Execute a tool on an MCP server. MCP tools define their own schemas and are registered dynamically.";

/// MCPTool - generic MCP tool execution dispatcher.
///
/// In TypeScript this is a template that gets overridden per MCP server/tool
/// by mcpClient.ts. In Rust, MCP tool execution is already handled via
/// McpToolRegistry in services/mcp/tool_executor.rs. This struct provides
/// a registry entry with an empty schema; actual MCP tool calls are dispatched
/// through the McpToolRegistry mechanism.
pub struct McpTool;

impl McpTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        MCP_TOOL_NAME
    }

    pub fn description(&self) -> &str {
        DESCRIPTION
    }

    pub fn user_facing_name(&self, _input: Option<&serde_json::Value>) -> String {
        "mcp (MCP Tool Execution)".to_string()
    }

    pub fn get_tool_use_summary(&self, input: Option<&serde_json::Value>) -> Option<String> {
        input.and_then(|inp| inp["server"].as_str().map(|s| {
            if let Some(tool) = inp["tool"].as_str() {
                format!("{}: {}", s, tool)
            } else {
                s.to_string()
            }
        }))
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
                    "description": "The MCP server name"
                },
                "tool": {
                    "type": "string",
                    "description": "The tool name to execute on the server"
                },
                "arguments": {
                    "type": "object",
                    "description": "Arguments to pass to the MCP tool"
                }
            }),
            required: Some(vec!["server".to_string(), "tool".to_string()]),
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

        let tool = input["tool"]
            .as_str()
            .ok_or_else(|| AgentError::Tool("Missing tool parameter".to_string()))?;

        let arguments = input.get("arguments").cloned().unwrap_or(serde_json::json!({}));

        // Dispatch through the global MCP tool registry
        crate::services::mcp::tool_executor::execute_mcp_tool(server, tool, arguments).await
    }
}

impl Default for McpTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_name() {
        let tool = McpTool::new();
        assert_eq!(tool.name(), MCP_TOOL_NAME);
    }

    #[test]
    fn test_mcp_tool_description() {
        let tool = McpTool::new();
        assert_eq!(tool.description(), DESCRIPTION);
    }

    #[test]
    fn test_mcp_tool_user_facing_name() {
        let tool = McpTool::new();
        assert_eq!(tool.user_facing_name(None), "mcp (MCP Tool Execution)");
    }

    #[test]
    fn test_mcp_tool_input_schema_has_properties() {
        let tool = McpTool::new();
        let schema = tool.input_schema();
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties["server"].is_object());
        assert!(schema.properties["tool"].is_object());
        assert!(schema.properties["arguments"].is_object());
    }

    #[tokio::test]
    async fn test_mcp_tool_missing_server() {
        let tool = McpTool::new();
        let result = tool.execute(serde_json::json!({"tool": "readFile"}), &ToolContext::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mcp_tool_missing_tool() {
        let tool = McpTool::new();
        let result = tool.execute(serde_json::json!({"server": "fs"}), &ToolContext::default()).await;
        assert!(result.is_err());
    }
}
