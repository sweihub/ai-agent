// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/types.ts
//! MCP types and configurations

use serde::{Deserialize, Serialize};

/// Configuration scope for MCP servers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConfigScope {
    Local,
    User,
    Project,
    Dynamic,
    Enterprise,
    ClaudeAi,
    Managed,
}

impl Default for ConfigScope {
    fn default() -> Self {
        Self::Local
    }
}

/// Transport type for MCP server connections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Transport {
    Stdio,
    Sse,
    SseIde,
    Http,
    Ws,
    Sdk,
}

/// MCP stdio server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpStdioServerConfig {
    #[serde(rename = "type", default)]
    pub config_type: Option<String>,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
}

/// MCP OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpOAuthConfig {
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub callback_port: Option<u16>,
    #[serde(default)]
    pub auth_server_metadata_url: Option<String>,
    #[serde(default)]
    pub xaa: Option<bool>,
}

/// MCP SSE server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSseServerConfig {
    #[serde(rename = "type")]
    pub config_type: String,
    pub url: String,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub headers_helper: Option<String>,
    #[serde(default)]
    pub oauth: Option<McpOAuthConfig>,
}

/// MCP SSE IDE server configuration (internal use for IDE extensions)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSseIdeServerConfig {
    #[serde(rename = "type")]
    pub config_type: String,
    pub url: String,
    pub ide_name: String,
    #[serde(default)]
    pub ide_running_in_windows: Option<bool>,
}

/// MCP WebSocket IDE server configuration (internal use for IDE extensions)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpWebSocketIdeServerConfig {
    #[serde(rename = "type")]
    pub config_type: String,
    pub url: String,
    pub ide_name: String,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub ide_running_in_windows: Option<bool>,
}

/// MCP HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpHttpServerConfig {
    #[serde(rename = "type")]
    pub config_type: String,
    pub url: String,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub headers_helper: Option<String>,
    #[serde(default)]
    pub oauth: Option<McpOAuthConfig>,
}

/// MCP WebSocket server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpWebSocketServerConfig {
    #[serde(rename = "type")]
    pub config_type: String,
    pub url: String,
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub headers_helper: Option<String>,
}

/// MCP SDK server configuration (internal use)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSdkServerConfig {
    #[serde(rename = "type")]
    pub config_type: String,
    pub name: String,
}

/// MCP Claude.ai proxy server configuration (internal use)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpClaudeAiProxyServerConfig {
    #[serde(rename = "type")]
    pub config_type: String,
    pub url: String,
    pub id: String,
}

/// MCP server configuration (discriminated union)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpServerConfig {
    Stdio(McpStdioServerConfig),
    Sse(McpSseServerConfig),
    SseIde(McpSseIdeServerConfig),
    WebSocketIde(McpWebSocketIdeServerConfig),
    Http(McpHttpServerConfig),
    WebSocket(McpWebSocketServerConfig),
    Sdk(McpSdkServerConfig),
    ClaudeAiProxy(McpClaudeAiProxyServerConfig),
}

/// Scoped MCP server configuration with scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopedMcpServerConfig {
    #[serde(flatten)]
    pub config: McpServerConfig,
    pub scope: ConfigScope,
    /// For plugin-provided servers: the providing plugin's source (e.g. 'slack@anthropic')
    #[serde(default)]
    pub plugin_source: Option<String>,
}

/// MCP JSON configuration file format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpJsonConfig {
    #[serde(default)]
    pub mcp_servers: std::collections::HashMap<String, McpServerConfig>,
}

/// Server capabilities from MCP server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    #[serde(default)]
    pub tools: Option<serde_json::Value>,
    #[serde(default)]
    pub resources: Option<serde_json::Value>,
    #[serde(default)]
    pub prompts: Option<serde_json::Value>,
    #[serde(default)]
    pub logging: Option<serde_json::Value>,
}

/// Connected MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedMcpServer {
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub capabilities: Option<ServerCapabilities>,
    #[serde(default)]
    pub server_info: Option<McpServerInfo>,
    #[serde(default)]
    pub instructions: Option<String>,
    pub config: ScopedMcpServerConfig,
}

/// MCP server info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

/// Failed MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedMcpServer {
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub config: ScopedMcpServerConfig,
    #[serde(default)]
    pub error: Option<String>,
}

/// MCP server that needs authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeedsAuthMcpServer {
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub config: ScopedMcpServerConfig,
}

/// Pending MCP server (connecting)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingMcpServer {
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub config: ScopedMcpServerConfig,
    #[serde(default)]
    pub reconnect_attempt: Option<u32>,
    #[serde(default)]
    pub max_reconnect_attempts: Option<u32>,
}

/// Disabled MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisabledMcpServer {
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub config: ScopedMcpServerConfig,
}

/// MCP server connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpServerConnection {
    Connected(ConnectedMcpServer),
    Failed(FailedMcpServer),
    NeedsAuth(NeedsAuthMcpServer),
    Pending(PendingMcpServer),
    Disabled(DisabledMcpServer),
}

/// Server resource from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResource {
    pub uri: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    pub server: String,
}

/// Serialized tool from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedTool {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_json_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub is_mcp: Option<bool>,
    /// Original unnormalized tool name from MCP server
    #[serde(default)]
    pub original_tool_name: Option<String>,
}

/// Serialized client state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedClient {
    pub name: String,
    #[serde(rename = "type")]
    pub client_type: String,
    #[serde(default)]
    pub capabilities: Option<ServerCapabilities>,
}

/// MCP CLI state for persistence
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct McpCliState {
    pub clients: Vec<SerializedClient>,
    #[serde(default)]
    pub configs: std::collections::HashMap<String, ScopedMcpServerConfig>,
    #[serde(default)]
    pub tools: Vec<SerializedTool>,
    #[serde(default)]
    pub resources: std::collections::HashMap<String, Vec<ServerResource>>,
    /// Maps normalized names to original names
    #[serde(default)]
    pub normalized_names: Option<std::collections::HashMap<String, String>>,
}
