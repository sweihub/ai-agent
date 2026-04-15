// Source: /data/home/swei/claudecode/openclaudecode/src/entrypoints/mcp.ts
//! MCP module - re-exports MCP types from types.rs

pub use crate::types::McpConnectionStatus;
pub use crate::types::McpHttpConfig;
pub use crate::types::McpServerConfig;
pub use crate::types::McpSseConfig;
pub use crate::types::McpStdioConfig;
pub use crate::types::McpTool;

/// MCP connection representation
#[derive(Debug, Clone)]
pub struct McpConnection {
    pub name: String,
    pub status: McpConnectionStatus,
    pub tools: Vec<crate::types::ToolDefinition>,
}

impl McpConnection {
    /// Close the MCP connection
    pub async fn close(&mut self) {
        self.status = McpConnectionStatus::Disconnected;
        self.tools.clear();
    }
}
