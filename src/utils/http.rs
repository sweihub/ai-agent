// Source: /data/home/swei/claudecode/openclaudecode/src/utils/http.ts
//! HTTP utility constants and helpers

use crate::constants::env::ai;
use std::collections::HashMap;

/// Get the user agent string for API requests
pub fn get_user_agent() -> String {
    let version = std::env::var(ai::VERSION).unwrap_or_else(|_| "unknown".to_string());
    let user_type = std::env::var(ai::USER_TYPE).unwrap_or_else(|_| "external".to_string());
    let entrypoint = std::env::var(ai::CODE_ENTRYPOINT).unwrap_or_else(|_| "cli".to_string());
    let agent_sdk_version = std::env::var(ai::AGENT_SDK_VERSION);
    let client_app = std::env::var(ai::AGENT_SDK_CLIENT_APP);

    let mut ua = format!("claude-cli/{} ({}, {}", version, user_type, entrypoint);

    if let Ok(v) = agent_sdk_version {
        ua.push_str(&format!(", agent-sdk/{}", v));
    }

    if let Ok(app) = client_app {
        ua.push_str(&format!(", client-app/{}", app));
    }

    // TODO: Add workload suffix from getWorkload()

    ua.push(')');
    ua
}

/// Get the user agent string for MCP requests
pub fn get_mcp_user_agent() -> String {
    let version = std::env::var(ai::VERSION).unwrap_or_else(|_| "unknown".to_string());

    let mut parts: Vec<String> = vec![];

    if let Ok(v) = std::env::var(ai::CODE_ENTRYPOINT) {
        parts.push(v);
    }
    if let Ok(v) = std::env::var(ai::AGENT_SDK_VERSION) {
        parts.push(format!("agent-sdk/{}", v));
    }
    if let Ok(v) = std::env::var(ai::AGENT_SDK_CLIENT_APP) {
        parts.push(format!("client-app/{}", v));
    }

    if parts.is_empty() {
        format!("claude-code/{}", version)
    } else {
        format!("claude-code/{} ({})", version, parts.join(", "))
    }
}

/// Get the user agent string for WebFetch requests
pub fn get_web_fetch_user_agent() -> String {
    // Claude-User is Anthropic's publicly documented agent for user-initiated fetches
    // The claude-code suffix lets site operators distinguish local CLI traffic
    format!(
        "Claude-User ({}; +https://support.anthropic.com/)",
        get_user_agent()
    )
}

/// Authentication headers for API requests
#[derive(Debug, Clone)]
pub struct AuthHeaders {
    /// Headers map
    pub headers: HashMap<String, String>,
    /// Error message if auth unavailable
    pub error: Option<String>,
}

/// Get authentication headers for API requests
/// Returns either OAuth headers for Max/Pro users or API key headers for regular users
pub fn get_auth_headers() -> AuthHeaders {
    // Check for OAuth token via env var (Max/Pro subscribers)
    if let Ok(token) = std::env::var("AI_CODE_OAUTH_TOKEN") {
        if !token.is_empty() {
            let mut headers = HashMap::new();
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
            headers.insert("anthropic-beta".to_string(), "oauth-mcp-servers-2025-01-16".to_string());
            return AuthHeaders { headers, error: None };
        }
    }

    // Fall back to API key env var for regular users
    if let Ok(api_key) = std::env::var("AI_AUTH_TOKEN") {
        if !api_key.is_empty() {
            let mut headers = HashMap::new();
            headers.insert("x-api-key".to_string(), api_key);
            return AuthHeaders { headers, error: None };
        }
    }

    AuthHeaders {
        headers: HashMap::new(),
        error: Some("No API key available".to_string()),
    }
}

/// Wrapper that handles OAuth 401 errors by force-refreshing the token and
/// retrying once. Addresses clock drift scenarios where the local expiration
/// check disagrees with the server.
///
/// The request closure is called again on retry, so it should re-read auth
/// (e.g., via get_auth_headers()) to pick up the refreshed token.
///
/// Note: Full implementation requires handleOAuth401Error from auth module.
/// SDK implementation forwards the request as-is (caller handles auth refresh).
///
/// # Arguments
/// * `request` - The async request closure to execute
/// * `_also_403_revoked` - Also retry on 403 with "OAuth token has been revoked" body (unused in SDK)
///
/// # Returns
/// The result of the wrapped request
pub async fn with_oauth_401_retry<T, R>(
    request: impl FnOnce() -> R,
    _also_403_revoked: Option<bool>,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    R: Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
{
    // SDK: Forward request as-is. Full retry-on-401 logic requires handleOAuth401Error
    // from auth module which has heavy dependencies. Callers should implement their own
    // retry logic using get_auth_headers() to pick up refreshed tokens.
    request().await
}
