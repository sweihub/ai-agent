// Session history fetching from remote API
// Translated from TypeScript: src/assistant/sessionHistory.ts
//
// Also includes bridge config functions translated from:
// openclaudecode/src/bridge/bridgeConfig.ts

use crate::constants::env::{ai, ai_code};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// =============================================================================
// BRIDGE CONFIG - translated from openclaudecode/src/bridge/bridgeConfig.ts
// =============================================================================

/// Ant-only dev override: AI_BRIDGE_OAUTH_TOKEN, else None.
pub fn get_bridge_token_override() -> Option<String> {
    if std::env::var(ai::USER_TYPE).ok().as_deref() == Some("ant") {
        std::env::var(ai::BRIDGE_OAUTH_TOKEN).ok()
    } else {
        None
    }
}

/// Ant-only dev override: AI_BRIDGE_BASE_URL, else None.
pub fn get_bridge_base_url_override() -> Option<String> {
    if std::env::var(ai::USER_TYPE).ok().as_deref() == Some("ant") {
        std::env::var(ai::BRIDGE_BASE_URL).ok()
    } else {
        None
    }
}

/// OAuth tokens stored in secure storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
    pub scopes: Vec<String>,
    #[serde(rename = "subscriptionType")]
    pub subscription_type: Option<String>,
    #[serde(rename = "rateLimitTier")]
    pub rate_limit_tier: Option<String>,
}

/// Get OAuth tokens from secure storage (keychain or file)
pub fn get_claude_ai_oauth_tokens() -> Option<OAuthTokens> {
    // Check for force-set OAuth token from environment variable
    if let Ok(token) = std::env::var(ai::OAUTH_TOKEN) {
        if !token.is_empty() {
            return Some(OAuthTokens {
                access_token: token,
                refresh_token: None,
                expires_at: None,
                scopes: vec!["user:inference".to_string()],
                subscription_type: None,
                rate_limit_tier: None,
            });
        }
    }

    if let Ok(token) = std::env::var(ai_code::OAUTH_TOKEN) {
        if !token.is_empty() {
            return Some(OAuthTokens {
                access_token: token,
                refresh_token: None,
                expires_at: None,
                scopes: vec!["user:inference".to_string()],
                subscription_type: None,
                rate_limit_tier: None,
            });
        }
    }

    // Try to read from secure storage
    if let Some(home) = dirs::home_dir() {
        // Try the new path: ~/.ai/oauth/tokens.json
        let ai_oauth_path = home.join(".ai").join("oauth").join("tokens.json");
        if let Ok(tokens) = read_oauth_tokens_from_path(&ai_oauth_path) {
            return Some(tokens);
        }

        // Fallback to old path: ~/.ai/oauth/tokens.json
        let claude_oauth_path = home.join(".ai").join("oauth").join("tokens.json");
        if let Ok(tokens) = read_oauth_tokens_from_path(&claude_oauth_path) {
            return Some(tokens);
        }
    }

    None
}

fn read_oauth_tokens_from_path(path: &PathBuf) -> Result<OAuthTokens, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let tokens: OAuthTokens = serde_json::from_str(&content)?;
    Ok(tokens)
}

/// Access token for bridge API calls: dev override first, then the OAuth
/// keychain. None means "not logged in".
pub fn get_bridge_access_token() -> Option<String> {
    // First check dev override
    if let Some(token) = get_bridge_token_override() {
        return Some(token);
    }

    // Then check OAuth tokens
    get_claude_ai_oauth_tokens().map(|t| t.access_token)
}

/// Base URL for bridge API calls: dev override first, then the production
/// OAuth config. Always returns a URL.
pub fn get_bridge_base_url() -> String {
    // First check dev override
    if let Some(url) = get_bridge_base_url_override() {
        return url;
    }

    // Then check OAuth config
    get_oauth_config().base_api_url
}

/// Get all bridge-related headers for API calls
pub fn get_bridge_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();

    if let Some(token) = get_bridge_access_token() {
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
    }

    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());

    headers
}

// =============================================================================
// SESSION HISTORY
// =============================================================================

pub const HISTORY_PAGE_SIZE: u32 = 100;

pub type SDKMessage = serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPage {
    pub events: Vec<SDKMessage>,
    #[serde(rename = "firstId")]
    pub first_id: Option<String>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
struct SessionEventsResponse {
    data: Vec<SDKMessage>,
    #[serde(rename = "has_more")]
    has_more: bool,
    #[serde(rename = "first_id")]
    first_id: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "last_id")]
    last_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HistoryAuthCtx {
    pub base_url: String,
    pub headers: HashMap<String, String>,
}

pub struct OauthConfig {
    pub base_api_url: String,
}

fn get_oauth_config() -> OauthConfig {
    if std::env::var(ai::USER_TYPE).ok().as_deref() == Some("ant") {
        if std::env::var(ai::USE_LOCAL_OAUTH)
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
        {
            let api = std::env::var(ai::CLAUDE_LOCAL_OAUTH_API_BASE)
                .unwrap_or_else(|_| "http://localhost:8000".to_string());
            return OauthConfig {
                base_api_url: api.trim_end_matches('/').to_string(),
            };
        }
        if std::env::var(ai::USE_STAGING_OAUTH)
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
        {
            return OauthConfig {
                base_api_url: "https://api-staging.anthropic.com".to_string(),
            };
        }
    }

    if let Ok(custom_url) = std::env::var(ai_code::CUSTOM_OAUTH_URL) {
        let base = custom_url.trim_end_matches('/').to_string();
        return OauthConfig { base_api_url: base };
    }

    OauthConfig {
        base_api_url: "https://api.anthropic.com".to_string(),
    }
}

pub async fn prepare_api_request() -> Result<(String, String), crate::AgentError> {
    let access_token = get_access_token()?;
    let org_uuid = get_org_uuid()?;
    Ok((access_token, org_uuid))
}

fn get_access_token() -> Result<String, crate::AgentError> {
    if let Ok(token) = std::env::var(ai_code::ACCESS_TOKEN) {
        if !token.is_empty() {
            return Ok(token);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let keychain_path = home.join(".ai").join("oauth").join("tokens.json");
        if let Ok(content) = std::fs::read_to_string(&keychain_path) {
            if let Ok(tokens) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(token) = tokens.get("accessToken").and_then(|t| t.as_str()) {
                    return Ok(token.to_string());
                }
            }
        }
    }

    Err(crate::AgentError::Auth(
        "Claude Code web sessions require authentication with a Claude.ai account. Please run /login to authenticate, or check your authentication status with /status.".to_string(),
    ))
}

fn get_org_uuid() -> Result<String, crate::AgentError> {
    if let Ok(org) = std::env::var(ai_code::ORG_UUID) {
        if !org.is_empty() {
            return Ok(org);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let settings_path = home.join(".ai").join("settings.json");
        if let Ok(content) = std::fs::read_to_string(&settings_path) {
            if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(org) = settings.get("orgUUID").and_then(|o| o.as_str()) {
                    return Ok(org.to_string());
                }
            }
        }
    }

    Err(crate::AgentError::Auth(
        "Organization UUID not found. Please authenticate with Claude Code.".to_string(),
    ))
}

pub fn get_oauth_headers(access_token: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", access_token),
    );
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
    headers
}

pub async fn create_history_auth_ctx(
    session_id: &str,
) -> Result<HistoryAuthCtx, crate::AgentError> {
    let (access_token, org_uuid) = prepare_api_request().await?;
    let oauth_config = get_oauth_config();

    let mut headers = get_oauth_headers(&access_token);
    headers.insert(
        "anthropic-beta".to_string(),
        "ccr-byoc-2025-07-29".to_string(),
    );
    headers.insert("x-organization-uuid".to_string(), org_uuid);

    let base_url = format!(
        "{}/v1/sessions/{}/events",
        oauth_config.base_api_url, session_id
    );

    Ok(HistoryAuthCtx { base_url, headers })
}

async fn fetch_page(
    ctx: &HistoryAuthCtx,
    params: &HashMap<String, serde_json::Value>,
    label: &str,
) -> Result<Option<HistoryPage>, crate::AgentError> {
    let client = reqwest::Client::new();

    let mut query_params: Vec<(&str, String)> = Vec::new();
    for (key, value) in params {
        query_params.push((key.as_str(), value.to_string()));
    }

    let mut header_map = HeaderMap::new();
    for (key, value) in &ctx.headers {
        if let (Ok(name), Ok(val)) = (key.parse::<HeaderName>(), value.parse::<HeaderValue>()) {
            header_map.insert(name, val);
        }
    }

    let resp = client
        .get(&ctx.base_url)
        .headers(header_map)
        .query(&query_params)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await;

    match resp {
        Ok(response) => {
            if response.status() == reqwest::StatusCode::OK {
                let data: SessionEventsResponse = response
                    .json()
                    .await
                    .map_err(|e| crate::AgentError::Http(e))?;

                Ok(Some(HistoryPage {
                    events: data.data,
                    first_id: data.first_id,
                    has_more: data.has_more,
                }))
            } else {
                log_for_debugging(&format!("[{}] HTTP {}", label, response.status()));
                Ok(None)
            }
        }
        Err(e) => {
            log_for_debugging(&format!("[{}] error: {}", label, e));
            Ok(None)
        }
    }
}

fn log_for_debugging(message: &str) {
    log::debug!("{}", message);
}

pub async fn fetch_latest_events(
    ctx: &HistoryAuthCtx,
    limit: u32,
) -> Result<Option<HistoryPage>, crate::AgentError> {
    let mut params = HashMap::new();
    params.insert("limit".to_string(), serde_json::json!(limit));
    params.insert("anchor_to_latest".to_string(), serde_json::json!(true));

    fetch_page(ctx, &params, "fetchLatestEvents").await
}

pub async fn fetch_older_events(
    ctx: &HistoryAuthCtx,
    before_id: &str,
    limit: u32,
) -> Result<Option<HistoryPage>, crate::AgentError> {
    let mut params = HashMap::new();
    params.insert("limit".to_string(), serde_json::json!(limit));
    params.insert("before_id".to_string(), serde_json::json!(before_id));

    fetch_page(ctx, &params, "fetchOlderEvents").await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_oauth_headers() {
        let token = "test_token";
        let headers = get_oauth_headers(token);

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer test_token".to_string())
        );
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            headers.get("anthropic-version"),
            Some(&"2023-06-01".to_string())
        );
    }

    #[test]
    fn test_history_page_default() {
        let page = HistoryPage {
            events: vec![],
            first_id: None,
            has_more: false,
        };

        assert_eq!(page.events.len(), 0);
        assert_eq!(page.first_id, None);
        assert_eq!(page.has_more, false);
    }
}
