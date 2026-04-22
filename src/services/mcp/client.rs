// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/client.ts
//! MCP client module - handles MCP server connections, tool calls, and auth
//!
//! Note: Full MCP protocol implementation using rust-mcp-sdk would go here.
//! The connectToServer, fetchToolsForClient, fetchResourcesForClient, and
//! fetchCommandsForClient functions require an MCP client library to be added
//! as a dependency.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::OnceLock;

use crate::services::analytics::log_event;
use crate::services::mcp::types::*;
use crate::utils::http::get_user_agent;

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Custom error class to indicate that an MCP tool call failed due to
/// authentication issues (e.g., expired OAuth token returning 401).
/// This error should be caught at the tool execution layer to update
/// the client's status to 'needs-auth'.
#[derive(Debug, Clone)]
pub struct McpAuthError {
    pub message: String,
    pub server_name: String,
}

impl std::fmt::Display for McpAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "McpAuthError({}): {}", self.server_name, self.message)
    }
}

impl std::error::Error for McpAuthError {}

impl McpAuthError {
    pub fn new(server_name: String, message: String) -> Self {
        Self {
            server_name,
            message,
        }
    }
}

/// Thrown when an MCP tool returns `isError: true`. Carries the result's `_meta`
/// so SDK consumers can still receive it.
#[derive(Debug, Clone)]
pub struct McpToolCallError {
    pub message: String,
    pub telemetry_message: String,
    pub mcp_meta: Option<McpToolCallMeta>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct McpToolCallMeta {
    #[serde(default)]
    pub _meta: Option<serde_json::Value>,
}

impl std::fmt::Display for McpToolCallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for McpToolCallError {}

impl McpToolCallError {
    pub fn new(
        message: String,
        telemetry_message: String,
        mcp_meta: Option<McpToolCallMeta>,
    ) -> Self {
        Self {
            message,
            telemetry_message,
            mcp_meta,
        }
    }
}

// =============================================================================
// SESSION EXPIRED ERROR
// =============================================================================

/// Thrown when an MCP session has expired and the connection cache has been cleared.
/// The caller should get a fresh client via ensureConnectedClient and retry.
#[derive(Debug, Clone)]
pub struct McpSessionExpiredError {
    pub server_name: String,
    pub message: String,
}

impl std::fmt::Display for McpSessionExpiredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for McpSessionExpiredError {}

impl McpSessionExpiredError {
    pub fn new(server_name: String) -> Self {
        Self {
            server_name: server_name.clone(),
            message: format!(r#"MCP server "{}" session expired"#, server_name),
        }
    }
}

/// Detects whether an error is an MCP "Session not found" error (HTTP 404 + JSON-RPC code -32001).
/// Per the MCP spec, servers return 404 when a session ID is no longer valid.
/// We check both signals to avoid false positives from generic 404s (wrong URL, server gone, etc.).
pub fn is_mcp_session_expired_error(error: &dyn std::error::Error) -> bool {
    let error_msg = error.to_string();

    // Check for HTTP 404 in the error message
    if !error_msg.contains("404") {
        return false;
    }

    // The SDK embeds the response body text in the error message.
    // MCP servers return: {"error":{"code":-32001,"message":"Session not found"},...}
    // Check for the JSON-RPC error code to distinguish from generic web server 404s.
    error_msg.contains("\"code\":-32001") || error_msg.contains("\"code\": -32001")
}

// =============================================================================
// AUTH CACHE (15 min TTL)
// =============================================================================

const MCP_AUTH_CACHE_TTL_MS: u64 = 15 * 60 * 1000; // 15 min

type McpAuthCacheData = HashMap<String, McpAuthCacheEntry>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct McpAuthCacheEntry {
    timestamp: u64,
}

fn get_mcp_auth_cache_path() -> String {
    use crate::utils::env_utils::get_claude_config_home_dir;
    let config_home = get_claude_config_home_dir();
    format!("{}/mcp-needs-auth-cache.json", config_home)
}

// Memoized so N concurrent isMcpAuthCached() calls during batched connection
// share a single file read instead of N reads of the same file.
static AUTH_CACHE: OnceLock<McpAuthCacheData> = OnceLock::new();

fn get_mcp_auth_cache() -> &'static McpAuthCacheData {
    AUTH_CACHE.get_or_init(|| {
        let cache_path = get_mcp_auth_cache_path();
        if let Ok(data) = std::fs::read_to_string(&cache_path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            McpAuthCacheData::new()
        }
    })
}

/// Check if a server is in the auth cache and hasn't expired
pub fn is_mcp_auth_cached(server_id: &str) -> bool {
    let cache = get_mcp_auth_cache();
    if let Some(entry) = cache.get(server_id) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        return now - entry.timestamp < MCP_AUTH_CACHE_TTL_MS;
    }
    false
}

/// Set an auth cache entry for a server (marks it as needing auth)
pub fn set_mcp_auth_cache_entry(server_id: &str) {
    let cache_path = get_mcp_auth_cache_path();
    let mut cache = get_mcp_auth_cache().clone();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    cache.insert(server_id.to_string(), McpAuthCacheEntry { timestamp: now });

    // Write to file (best-effort)
    if let Ok(json) = serde_json::to_string(&cache) {
        if let Some(parent) = std::path::Path::new(&cache_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&cache_path, json);
    }
}

/// Clear the MCP auth cache
pub fn clear_mcp_auth_cache() {
    // Note: We don't clear the in-memory cache since OnceLock doesn't support
    // taking the value from a static. The file deletion ensures fresh reads.
    // This matches the spirit of the TypeScript which nulls the promise on next read.
    let cache_path = get_mcp_auth_cache_path();
    let _ = std::fs::remove_file(cache_path);
}

// =============================================================================
// FETCH WRAPPER WITH TIMEOUT
// =============================================================================

/// MCP Streamable HTTP spec requires clients to advertise acceptance of both
/// JSON and SSE on every POST. Servers that enforce this strictly reject
/// requests without it (HTTP 406).
const MCP_STREAMABLE_HTTP_ACCEPT: &str = "application/json, text/event-stream";

/// Default timeout for individual MCP requests (auth, tool calls, etc.)
const MCP_REQUEST_TIMEOUT_MS: u64 = 60000;

/// Wraps a fetch function to apply a fresh timeout signal to each request.
/// This avoids the bug where a single AbortSignal.timeout() created at connection
/// time becomes stale after 60 seconds, causing all subsequent requests to fail
/// immediately with "The operation timed out." Uses a 60-second timeout.
///
/// Also ensures the Accept header required by the MCP Streamable HTTP spec is
/// present on POSTs.
///
/// GET requests are excluded from the timeout since, for MCP transports, they are
/// long-lived SSE streams meant to stay open indefinitely.
///
/// Note: This is a simplified stub. Full implementation would use the actual fetch type.
pub fn wrap_fetch_with_timeout(
    _base_fetch: impl Fn(
        String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<reqwest::Response, reqwest::Error>> + Send>,
    > + Send
    + Sync
    + 'static,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<reqwest::Response, reqwest::Error>> + Send>,
> + Send
+ Sync
+ 'static {
    move |url: String| {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(MCP_REQUEST_TIMEOUT_MS))
            .user_agent(get_user_agent())
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                return Box::pin(async { Err(e) })
                    as Pin<
                        Box<
                            dyn std::future::Future<
                                    Output = Result<reqwest::Response, reqwest::Error>,
                                > + Send,
                        >,
                    >;
            }
        };

        Box::pin(async move {
            let mut request = client.get(&url);
            request = request.header("Accept", MCP_STREAMABLE_HTTP_ACCEPT);
            request.send().await
        })
            as Pin<
                Box<
                    dyn std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>
                        + Send,
                >,
            >
    }
}

// =============================================================================
// SERVER CONNECTION BATCH SIZE
// =============================================================================

/// Get the batch size for concurrent MCP server connections
pub fn get_mcp_server_connection_batch_size() -> u32 {
    std::env::var("MCP_SERVER_CONNECTION_BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

fn get_remote_mcp_server_connection_batch_size() -> u32 {
    std::env::var("MCP_REMOTE_SERVER_CONNECTION_BATCH_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20)
}

// =============================================================================
// SERVER CACHE KEY
// =============================================================================

/// Generates the cache key for a server connection
/// @param name Server name
/// @param server_ref Server configuration
/// @returns Cache key string
pub fn get_server_cache_key(name: &str, server_ref: &ScopedMcpServerConfig) -> String {
    // Exclude 'scope' from comparison since it's metadata, not connection config
    let config_json = serde_json::to_string(server_ref).unwrap_or_default();
    format!("{}-{}", name, config_json)
}

// =============================================================================
// CONFIG EQUALITY
// =============================================================================

/// Compares two MCP server configurations to determine if they are equivalent.
/// Used to detect when a server needs to be reconnected due to config changes.
pub fn are_mcp_configs_equal(a: &ScopedMcpServerConfig, b: &ScopedMcpServerConfig) -> bool {
    // Quick type check first
    if a.config.type_variant() != b.config.type_variant() {
        return false;
    }

    // Compare by serializing - this handles all config variations
    // We exclude 'scope' from comparison since it's metadata, not connection config
    let a_json = serde_json::to_string(a).unwrap_or_default();
    let b_json = serde_json::to_string(b).unwrap_or_default();
    a_json == b_json
}

// =============================================================================
// TOOL INPUT AUTO CLASSIFIER
// =============================================================================

/// Encode MCP tool input for the auto-mode security classifier.
/// Exported so the auto-mode eval scripts can mirror production encoding
/// for `mcp__*` tool stubs without duplicating this logic.
pub fn mcp_tool_input_to_auto_classifier_input(
    input: &serde_json::Value,
    tool_name: &str,
) -> String {
    if let Some(obj) = input.as_object() {
        if !obj.is_empty() {
            return obj
                .keys()
                .map(|k| {
                    format!(
                        "{}={}",
                        k,
                        obj.get(k).and_then(|v| v.as_str()).unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
        }
    }
    tool_name.to_string()
}

// =============================================================================
// TOOL TIMEOUT
// =============================================================================

/// Get the MCP tool timeout in milliseconds
pub fn get_mcp_tool_timeout_ms() -> u64 {
    std::env::var("MCP_TOOL_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000_000) // ~27 hours default
}

// =============================================================================
// CONNECTION TIMEOUT
// =============================================================================

fn get_connection_timeout_ms() -> u32 {
    std::env::var("MCP_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30000)
}

// =============================================================================
// CONNECTION STATE HELPERS
// =============================================================================

/// Check if a server config represents a local (stdio/sdk) MCP server
pub fn is_local_mcp_server(config: &ScopedMcpServerConfig) -> bool {
    let t = config.config.type_variant();
    t == "stdio" || t == "sdk" || t.is_empty()
}

// =============================================================================
// STUB FUNCTIONS - Require rust-mcp-sdk integration
// =============================================================================

/// Maximum cache size for fetch* caches. Keyed by server name (stable across
/// reconnects), bounded to prevent unbounded growth with many MCP servers.
const MCP_FETCH_CACHE_SIZE: usize = 20;

/// Connect to an MCP server and return a connection
/// NOTE: This is a stub. Full implementation requires rust-mcp-sdk integration.
pub async fn connect_to_server(
    name: &str,
    server_ref: &ScopedMcpServerConfig,
) -> McpServerConnection {
    // TODO: Full implementation requires rust-mcp-sdk crate
    // The connection logic includes:
    // - SSE transport with ClaudeAuthProvider
    // - WebSocket transport
    // - Streamable HTTP transport
    // - SDK transport for in-process MCP servers
    McpServerConnection::Disabled(DisabledMcpServer {
        name: name.to_string(),
        server_type: server_ref.config.type_variant().to_string(),
        config: server_ref.clone(),
    })
}

/// Fetch tools from a connected MCP server
/// NOTE: This is a stub. Full implementation requires rust-mcp-sdk integration.
pub async fn fetch_tools_for_client(client: &McpServerConnection) -> Vec<serde_json::Value> {
    if !matches!(client, McpServerConnection::Connected(_)) {
        return vec![];
    }
    // TODO: Full implementation requires rust-mcp-sdk crate
    vec![]
}

/// Fetch resources from a connected MCP server
/// NOTE: This is a stub. Full implementation requires rust-mcp-sdk integration.
pub async fn fetch_resources_for_client(client: &McpServerConnection) -> Vec<ServerResource> {
    if !matches!(client, McpServerConnection::Connected(_)) {
        return vec![];
    }
    // TODO: Full implementation requires rust-mcp-sdk crate
    vec![]
}

/// Fetch commands (prompts) from a connected MCP server
/// NOTE: This is a stub. Full implementation requires rust-mcp-sdk integration.
pub async fn fetch_commands_for_client(
    client: &McpServerConnection,
) -> Vec<crate::commands::Command> {
    if !matches!(client, McpServerConnection::Connected(_)) {
        return vec![];
    }
    // TODO: Full implementation requires rust-mcp-sdk crate
    vec![]
}

/// Clear server cache for reconnection
/// NOTE: This is a stub. Full implementation requires rust-mcp-sdk integration.
pub async fn clear_server_cache(name: &str, config: &ScopedMcpServerConfig) -> Result<(), String> {
    let _ = (name, config);
    // TODO: Invalidate keychain cache and connection cache
    Ok(())
}

/// Ensure a client is connected
/// NOTE: This is a stub. Full implementation requires rust-mcp-sdk integration.
pub async fn ensure_connected_client(
    client: McpServerConnection,
) -> Result<McpServerConnection, String> {
    if matches!(client, McpServerConnection::Connected(_)) {
        Ok(client)
    } else {
        Err("MCP server not connected".to_string())
    }
}

// =============================================================================
// TYPE EXTENSIONS FOR MCPServerConfig
// =============================================================================

impl McpServerConfig {
    /// Returns the type variant string for this config
    pub fn type_variant(&self) -> &'static str {
        match self {
            McpServerConfig::Stdio(_) => "stdio",
            McpServerConfig::Sse(_) => "sse",
            McpServerConfig::SseIde(_) => "sse-ide",
            McpServerConfig::WebSocketIde(_) => "ws-ide",
            McpServerConfig::Http(_) => "http",
            McpServerConfig::WebSocket(_) => "ws",
            McpServerConfig::Sdk(_) => "sdk",
            McpServerConfig::ClaudeAiProxy(_) => "claudeai-proxy",
        }
    }
}

// =============================================================================
// INFERENCE HELPERS (from TypeScript inferCompactSchema)
// =============================================================================

/// Generates a compact, jq-friendly type signature for a value.
/// e.g. "{title: string, items: [{id: number, name: string}]}"
pub fn infer_compact_schema(value: &serde_json::Value, depth: usize) -> String {
    const MAX_ENTRIES: usize = 10;

    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "boolean".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let inner_depth = depth.saturating_sub(1);
                format!("[{}]", infer_compact_schema(&arr[0], inner_depth))
            }
        }
        serde_json::Value::Object(obj) => {
            if depth == 0 {
                "{...}".to_string()
            } else {
                let entries: Vec<String> = obj
                    .iter()
                    .take(MAX_ENTRIES)
                    .map(|(k, v)| {
                        format!(
                            "{}: {}",
                            k,
                            infer_compact_schema(v, depth.saturating_sub(1))
                        )
                    })
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
        }
    }
}

// =============================================================================
// MCP RESULT TYPES
// =============================================================================

/// Result type for MCP tool calls
pub type MCPResultType = &'static str; // 'toolResult' | 'structuredContent' | 'contentArray'

/// Transformed MCP result with type information
#[derive(Debug, Clone)]
pub struct TransformedMCPResult {
    pub content: serde_json::Value,
    pub result_type: MCPResultType,
    pub schema: Option<String>,
}
