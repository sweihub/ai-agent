// Source: /data/home/swei/claudecode/openclaudecode/src/types/generated/events_mono/common/v1/auth.ts
//! MCP Authentication module
//!
//! Provides OAuth flow support for MCP servers that require authentication.
//! SDK users register an OAuth callback via `register_mcp_oauth_callback()`.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// MCP authentication config
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub auth_type: Option<String>,
    pub token: Option<String>,
}

/// MCP OAuth configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub scopes: Vec<String>,
}

/// Get MCP auth headers
pub fn get_auth_headers(config: &AuthConfig) -> std::collections::HashMap<String, String> {
    let mut headers = std::collections::HashMap::new();

    if let Some(token) = &config.token {
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
    }

    headers
}

/// Check if auth is required for an MCP server
pub fn is_auth_required(config: &AuthConfig) -> bool {
    config.enabled && config.auth_type.is_some()
}

// ============================================================================
// MCP OAuth flow — matches TypeScript performMCPOAuthFlow
// ============================================================================

/// Result of starting the MCP OAuth flow
#[derive(Debug, Clone)]
pub struct McpOAuthResult {
    /// Status of the auth flow
    pub status: McpOAuthStatus,
    /// Human-readable message for the user
    pub message: String,
    /// Authorization URL to share with the user (if status is AuthUrl)
    pub auth_url: Option<String>,
}

/// Status of the MCP OAuth flow
#[derive(Debug, Clone, PartialEq)]
pub enum McpOAuthStatus {
    /// An authorization URL was returned; the user needs to open it
    AuthUrl,
    /// The auth completed silently (e.g., cached token)
    Authenticated,
    /// OAuth is not supported for this transport
    Unsupported,
    /// An error occurred during the flow
    Error,
}

/// Callback type for performing MCP OAuth flow.
/// Takes (server_name, config_json, on_auth_url_callback).
pub type McpOAuthCallback = Arc<
    dyn Fn(
        String,
        serde_json::Value,
        Option<Arc<dyn Fn(String) + Send + Sync>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<McpOAuthResult, crate::AgentError>> + Send + Sync>>
        + Send
        + Sync,
>;

/// Global OAuth callback registered by the SDK user.
static MCP_OAUTH_CALLBACK: once_cell::sync::Lazy<parking_lot::RwLock<Option<McpOAuthCallback>>> =
    once_cell::sync::Lazy::new(Default::default);

/// Register a callback for MCP OAuth flow.
///
/// The callback is invoked when a server that requires OAuth is used.
/// It should start the OAuth flow and return the authorization URL.
pub fn register_mcp_oauth_callback<F, Fut>(callback: F)
where
    F: Fn(
        String,
        serde_json::Value,
        Option<Arc<dyn Fn(String) + Send + Sync>>,
    ) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<McpOAuthResult, crate::AgentError>> + Send + Sync + 'static,
{
    let wrapped: McpOAuthCallback = Arc::new(
        move |server: String, config: serde_json::Value, on_url: Option<Arc<dyn Fn(String) + Send + Sync>>| {
            Box::pin(callback(server, config, on_url))
        },
    );
    *MCP_OAUTH_CALLBACK.write() = Some(wrapped);
}

/// Execute the MCP OAuth flow for a server.
///
/// Returns an error if no OAuth callback has been registered.
pub async fn perform_mcp_oauth_flow(
    server_name: String,
    config: serde_json::Value,
    on_auth_url: Option<Arc<dyn Fn(String) + Send + Sync>>,
) -> Result<McpOAuthResult, crate::AgentError> {
    let callback = MCP_OAUTH_CALLBACK.read().clone();
    match callback {
        Some(cb) => cb(server_name, config, on_auth_url).await,
        None => Err(crate::AgentError::Tool(
            "No MCP OAuth callback registered. Call register_mcp_oauth_callback() to enable OAuth.".to_string(),
        )),
    }
}

// Re-export for use by McpAuthTool (accessed via crate::services::mcp::client::clear_mcp_auth_cache)
