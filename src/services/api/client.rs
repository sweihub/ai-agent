// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/client.ts
//! Anthropic API client
//! Creates and configures the Anthropic SDK client for different providers

use std::collections::HashMap;
use std::env;

/// Client request ID header name
pub const CLIENT_REQUEST_ID_HEADER: &str = "x-client-request-id";

/// Check if env var is truthy
fn is_env_truthy(value: Option<String>) -> bool {
    value
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get API timeout in milliseconds
fn get_api_timeout_ms() -> u64 {
    env::var("AI_CODE_API_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(600_000)
}

/// Get session ID
fn get_session_id() -> String {
    env::var("AI_CODE_SESSION_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string())
}

/// Get container ID if set
fn get_container_id() -> Option<String> {
    env::var("AI_CODE_CONTAINER_ID").ok()
}

/// Get remote session ID if set
fn get_remote_session_id() -> Option<String> {
    env::var("AI_CODE_REMOTE_SESSION_ID").ok()
}

/// Get client app if set
fn get_client_app() -> Option<String> {
    env::var("AI_AGENT_SDK_CLIENT_APP").ok()
}

/// Get custom headers from environment
fn get_custom_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    if let Ok(custom_headers_env) = env::var("AI_CODE_CUSTOM_HEADERS") {
        if custom_headers_env.is_empty() {
            return headers;
        }
        for header_string in custom_headers_env.lines() {
            let header_string = header_string.trim();
            if header_string.is_empty() {
                continue;
            }
            if let Some(colon_idx) = header_string.find(':') {
                let name = header_string[..colon_idx].trim().to_string();
                let value = header_string[colon_idx + 1..].trim().to_string();
                if !name.is_empty() {
                    headers.insert(name, value);
                }
            }
        }
    }
    headers
}

/// Check additional protection is enabled
fn is_additional_protection_enabled() -> bool {
    is_env_truthy(env::var("AI_CODE_ADDITIONAL_PROTECTION").ok())
}

/// Check if using Bedrock
fn is_using_bedrock() -> bool {
    is_env_truthy(env::var("AI_CODE_USE_BEDROCK").ok())
}

/// Check if using Foundry (Azure)
fn is_using_foundry() -> bool {
    is_env_truthy(env::var("AI_CODE_USE_FOUNDRY").ok())
}

/// Check if using Vertex
fn is_using_vertex() -> bool {
    is_env_truthy(env::var("AI_CODE_USE_VERTEX").ok())
}

/// Get AWS region
fn get_aws_region() -> String {
    env::var("AI_CODE_AWS_REGION")
        .or_else(|_| env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string())
}

/// Get small fast model name
fn get_small_fast_model() -> String {
    // Default to Haiku
    "claude-3-5-haiku-20241022".to_string()
}

/// Get Bearer token for Bedrock if set
fn get_bearer_token_bedrock() -> Option<String> {
    env::var("AWS_BEARER_TOKEN_BEDROCK").ok()
}

/// Check if skipping Bedrock auth
fn is_skipping_bedrock_auth() -> bool {
    is_env_truthy(env::var("AI_CODE_SKIP_BEDROCK_AUTH").ok())
}

/// Check if skipping Vertex auth
fn is_skipping_vertex_auth() -> bool {
    is_env_truthy(env::var("AI_CODE_SKIP_VERTEX_AUTH").ok())
}

/// Get Vertex project ID
fn get_vertex_project_id() -> Option<String> {
    env::var("ANTHROPIC_VERTEX_PROJECT_ID").ok()
}

/// Get vertex region for model
fn get_vertex_region_for_model(_model: Option<&str>) -> String {
    // Default region
    "us-east5".to_string()
}

/// Check GCP project env vars
fn has_project_env_var() -> bool {
    env::var("GCLOUD_PROJECT").is_ok()
        || env::var("GOOGLE_CLOUD_PROJECT").is_ok()
        || env::var("gcloud_project").is_ok()
        || env::var("google_cloud_project").is_ok()
}

/// Check for credential file paths
fn has_key_file() -> bool {
    env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok()
        || env::var("google_application_credentials").is_ok()
}

/// Check if user type is 'ant'
fn is_ant_user() -> bool {
    env::var("AI_CODE_USER_TYPE")
        .map(|v| v == "ant")
        .unwrap_or(false)
}

/// Check if using staging OAuth
fn is_using_staging_oauth() -> bool {
    is_env_truthy(env::var("USE_STAGING_OAUTH").ok())
}

/// Get base API URL for OAuth
fn get_oauth_base_api_url() -> String {
    // Default staging URL
    env::var("AI_CODE_API_URL").unwrap_or_else(|_| "https://api.staging.anthropic.com".to_string())
}

/// Check if debug to stderr is enabled
fn is_debug_to_stderr() -> bool {
    is_env_truthy(env::var("AI_CODE_DEBUG_TO_STDERR").ok())
}

/// Create default headers for API client
pub fn create_default_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("x-app".to_string(), "cli".to_string());
    headers.insert(
        "User-Agent".to_string(),
        format!("ai-agent/{}", env!("CARGO_PKG_VERSION")),
    );
    headers.insert("X-Claude-Code-Session-Id".to_string(), get_session_id());

    // Add custom headers
    for (k, v) in get_custom_headers() {
        headers.insert(k, v);
    }

    // Container ID
    if let Some(container_id) = get_container_id() {
        headers.insert("x-claude-remote-container-id".to_string(), container_id);
    }

    // Remote session ID
    if let Some(remote_session_id) = get_remote_session_id() {
        headers.insert("x-claude-remote-session-id".to_string(), remote_session_id);
    }

    // Client app
    if let Some(client_app) = get_client_app() {
        headers.insert("x-client-app".to_string(), client_app);
    }

    // Additional protection
    if is_additional_protection_enabled() {
        headers.insert(
            "x-anthropic-additional-protection".to_string(),
            "true".to_string(),
        );
    }

    headers
}

/// Get API key from environment or helper
fn get_anthropic_api_key() -> Option<String> {
    env::var("AI_CODE_API_KEY")
        .ok()
        .or_else(|| env::var("ANTHROPIC_API_KEY").ok())
}

/// Check if Claude.ai subscriber
fn is_claude_ai_subscriber() -> bool {
    // Check for OAuth token which indicates a subscriber
    env::var("AI_CODE_OAUTH_TOKEN").is_ok()
}

/// Get Claude.ai OAuth tokens
fn get_claudeai_oauth_tokens() -> Option<OAuthTokens> {
    env::var("AI_CODE_OAUTH_TOKEN")
        .ok()
        .map(|token| OAuthTokens {
            access_token: token,
        })
}

#[derive(Debug, Clone)]
pub struct OAuthTokens {
    pub access_token: String,
}

/// Check and refresh OAuth token if needed
async fn check_and_refresh_oauth_token_if_needed() {
    // For now, just log that we're checking
    log::debug!("[API:auth] OAuth token check");
    // The actual refresh is handled by the auth module
}

/// Configure API key headers
async fn configure_api_key_headers(
    headers: &mut HashMap<String, String>,
    _is_non_interactive: bool,
) {
    if let Ok(token) = env::var("ANTHROPIC_AUTH_TOKEN") {
        if !token.is_empty() {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
    }
}

/// Anthropic client configuration
#[derive(Debug, Clone)]
pub struct AnthropicClientConfig {
    pub api_key: Option<String>,
    pub auth_token: Option<String>,
    pub base_url: Option<String>,
    pub max_retries: u32,
    pub timeout: u64,
}

/// Get Anthropic client (simplified implementation)
/// In full implementation, this would support Bedrock, Foundry, and Vertex
pub async fn get_anthropic_client(
    config: AnthropicClientConfig,
) -> Result<reqwest::Client, String> {
    let mut default_headers = create_default_headers();

    // OAuth token check
    check_and_refresh_oauth_token_if_needed().await;

    // Configure headers for non-subscribers
    if !is_claude_ai_subscriber() {
        configure_api_key_headers(&mut default_headers, false).await;
    }

    // Convert headers to HeaderMap
    let mut header_map = reqwest::header::HeaderMap::new();
    for (k, v) in default_headers {
        if let (Ok(name), Ok(value)) = (
            k.parse::<reqwest::header::HeaderName>(),
            v.parse::<reqwest::header::HeaderValue>(),
        ) {
            header_map.insert(name, value);
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(config.timeout))
        .default_headers(header_map)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(client)
}

/// Build the API client based on configuration
pub async fn build_anthropic_client(
    api_key: Option<String>,
    max_retries: u32,
    _model: Option<String>,
    _source: Option<String>,
) -> Result<reqwest::Client, String> {
    let config = AnthropicClientConfig {
        api_key: if is_claude_ai_subscriber() {
            None
        } else {
            api_key.or_else(get_anthropic_api_key)
        },
        auth_token: if is_claude_ai_subscriber() {
            get_claudeai_oauth_tokens().map(|t| t.access_token)
        } else {
            None
        },
        base_url: if is_ant_user() && is_using_staging_oauth() {
            Some(get_oauth_base_api_url())
        } else {
            None
        },
        max_retries,
        timeout: get_api_timeout_ms(),
    };

    get_anthropic_client(config).await
}

/// Get proxy fetch options (simplified)
pub fn get_proxy_fetch_options(_for_anthropic_api: bool) -> HashMap<String, String> {
    // TODO: Integrate with proxy system
    HashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_env_truthy() {
        assert!(!is_env_truthy(None));
        assert!(!is_env_truthy(Some("0".to_string())));
        assert!(!is_env_truthy(Some("false".to_string())));
        assert!(is_env_truthy(Some("1".to_string())));
        assert!(is_env_truthy(Some("true".to_string())));
    }

    #[test]
    fn test_create_default_headers_has_required() {
        let headers = create_default_headers();
        assert!(headers.contains_key("x-app"));
        assert!(headers.contains_key("User-Agent"));
        assert!(headers.contains_key("X-Claude-Code-Session-Id"));
    }

    #[test]
    fn test_get_api_timeout_default() {
        // Without env var, should return default
        let timeout = get_api_timeout_ms();
        assert_eq!(timeout, 600_000);
    }
}
