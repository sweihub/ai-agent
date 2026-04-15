// Source: ~/claudecode/openclaudecode/src/hooks/useIdeConnectionStatus.ts
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// The connection status of the IDE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeStatus {
    Connected,
    Disconnected,
    Pending,
}

/// Result of checking IDE connection status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeConnectionResult {
    pub status: Option<IdeStatus>,
    pub ide_name: Option<String>,
}

/// An MCP client config variant.
#[derive(Debug, Clone)]
pub enum McpClientConfig {
    SseIde { ide_name: String },
    WsIde { ide_name: String },
    Other,
}

/// An MCP client connection.
#[derive(Debug, Clone)]
pub struct McpClientConnection {
    pub name: String,
    pub client_type: McpClientType,
    pub config: McpClientConfig,
}

/// The type/state of an MCP client connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpClientType {
    Connected,
    Pending,
    Disconnected,
}

/// Check the IDE connection status from a list of MCP clients.
///
/// Translation of the React `useIdeConnectionStatus` hook.
/// In Rust this is a plain function (equivalent to `useMemo` with dependencies).
pub fn get_ide_connection_status(
    mcp_clients: Option<&[McpClientConnection]>,
) -> IdeConnectionResult {
    let Some(clients) = mcp_clients else {
        return IdeConnectionResult {
            status: None,
            ide_name: None,
        };
    };

    let ide_client = clients.iter().find(|c| c.name == "ide");
    let Some(ide_client) = ide_client else {
        return IdeConnectionResult {
            status: None,
            ide_name: None,
        };
    };

    // Extract IDE name from config if available.
    let ide_name = match &ide_client.config {
        McpClientConfig::SseIde { ide_name } | McpClientConfig::WsIde { ide_name } => {
            Some(ide_name.clone())
        }
        McpClientConfig::Other => None,
    };

    let status = match ide_client.client_type {
        McpClientType::Connected => Some(IdeStatus::Connected),
        McpClientType::Pending => Some(IdeStatus::Pending),
        McpClientType::Disconnected => Some(IdeStatus::Disconnected),
    };

    IdeConnectionResult { status, ide_name }
}
