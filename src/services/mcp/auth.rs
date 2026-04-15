// Source: /data/home/swei/claudecode/openclaudecode/src/types/generated/events_mono/common/v1/auth.ts
//! MCP Authentication module

use serde::{Deserialize, Serialize};

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
