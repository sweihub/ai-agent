// Source: /data/home/swei/claudecode/openclaudecode/src/services/oauth/client.ts
//! OAuth client helpers for building URLs, exchanging codes, refreshing tokens,
//! and fetching user profile information.
//!
//! Source: /data/home/swei/claudecode/openclaudecode/src/services/oauth/crypto.ts

use crate::services::oauth::constants::{get_oauth_config, get_client_id};
use crate::services::oauth::types::{
    OAuthProfileResponse, OAuthTokenExchangeResponse, OAuthTokens, RateLimitTier,
    SubscriptionType, TokenAccount, UserRolesResponse, CLAUDE_AI_INFERENCE_SCOPE,
    CLAUDE_AI_OAUTH_SCOPES,
};

use std::collections::HashMap;

/// Check if the user has Claude.ai authentication scope.
/// Only call this from OAuth / auth related code!
pub fn should_use_claude_ai_auth(scopes: &[&str]) -> bool {
    scopes.contains(&CLAUDE_AI_INFERENCE_SCOPE)
}

pub fn parse_scopes(scope_string: Option<&str>) -> Vec<String> {
    scope_string
        .unwrap_or("")
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Build the OAuth authorization URL.
pub fn build_auth_url(
    code_challenge: &str,
    state: &str,
    port: u16,
    is_manual: bool,
    login_with_claude_ai: bool,
    inference_only: bool,
    org_uuid: Option<&str>,
    login_hint: Option<&str>,
    login_method: Option<&str>,
) -> String {
    let config = get_oauth_config();
    let auth_url_base = if login_with_claude_ai {
        &config.claude_ai_authorize_url
    } else {
        &config.console_authorize_url
    };

    let mut url = format!("{auth_url_base}?");

    // This tells the login page to show Claude Max upsell
    url.push_str("code=true&");
    url.push_str(&format!("client_id={}&", get_client_id()));
    url.push_str("response_type=code&");

    let redirect_uri = if is_manual {
        config.manual_redirect_url.clone()
    } else {
        format!("http://localhost:{port}/callback")
    };
    url.push_str(&format!("redirect_uri={}&", urlencoding::encode(&redirect_uri)));

    let scopes_to_use = if inference_only {
        vec![CLAUDE_AI_INFERENCE_SCOPE]
    } else {
        crate::services::oauth::types::all_oauth_scopes()
    };
    url.push_str(&format!("scope={}&", urlencoding::encode(&scopes_to_use.join(" "))));
    url.push_str(&format!("code_challenge={}&", urlencoding::encode(code_challenge)));
    url.push_str("code_challenge_method=S256&");
    url.push_str(&format!("state={}&", urlencoding::encode(state)));

    if let Some(uuid) = org_uuid {
        url.push_str(&format!("orgUUID={}&", urlencoding::encode(uuid)));
    }
    if let Some(hint) = login_hint {
        url.push_str(&format!("login_hint={}&", urlencoding::encode(hint)));
    }
    if let Some(method) = login_method {
        url.push_str(&format!("login_method={}&", urlencoding::encode(method)));
    }

    url
}

/// Exchange an authorization code for OAuth tokens using the PKCE flow.
pub async fn exchange_code_for_tokens(
    authorization_code: String,
    state: String,
    code_verifier: String,
    port: u16,
    use_manual_redirect: bool,
    expires_in: Option<u64>,
) -> anyhow::Result<OAuthTokenExchangeResponse> {
    let config = get_oauth_config();
    let redirect_uri = if use_manual_redirect {
        config.manual_redirect_url.clone()
    } else {
        format!("http://localhost:{port}/callback")
    };

    let mut body: HashMap<String, serde_json::Value> = HashMap::new();
    body.insert("grant_type".to_string(), "authorization_code".into());
    body.insert("code".to_string(), authorization_code.into());
    body.insert("redirect_uri".to_string(), redirect_uri.into());
    body.insert("client_id".to_string(), get_client_id().into());
    body.insert("code_verifier".to_string(), code_verifier.into());
    body.insert("state".to_string(), state.into());

    if let Some(ei) = expires_in {
        body.insert("expires_in".to_string(), ei.into());
    }

    let client = reqwest::Client::builder()
        .user_agent(crate::utils::user_agent::get_user_agent())
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let response = client
        .post(&config.token_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if response.status() != 200 {
        let status = response.status().as_u16();
        let status_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            if status == 401 {
                "Authentication failed: Invalid authorization code"
            } else {
                "Token exchange failed ({status}): {status_text}"
            }
        ));
    }

    log::info!("OAuth token exchange succeeded");
    let data: OAuthTokenExchangeResponse = response.json().await?;
    Ok(data)
}

/// Refresh an OAuth token using the refresh grant type.
pub async fn refresh_oauth_token(
    refresh_token: String,
    scopes: Option<Vec<String>>,
) -> anyhow::Result<OAuthTokens> {
    let config = get_oauth_config();
    let requested_scopes = scopes.unwrap_or_else(|| CLAUDE_AI_OAUTH_SCOPES.iter().map(|s| s.to_string()).collect());

    let mut body: HashMap<String, serde_json::Value> = HashMap::new();
    body.insert("grant_type".to_string(), "refresh_token".into());
    body.insert("refresh_token".to_string(), refresh_token.clone().into());
    body.insert("client_id".to_string(), get_client_id().into());
    body.insert("scope".to_string(), requested_scopes.join(" ").into());

    let client = reqwest::Client::builder()
        .user_agent(crate::utils::user_agent::get_user_agent())
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let response = client
        .post(&config.token_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if response.status() != 200 {
        let status_text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Token refresh failed: {status_text}"));
    }

    let data: OAuthTokenExchangeResponse = response.json().await?;
    let access_token = data.access_token;
    let new_refresh_token = data.refresh_token.unwrap_or_else(|| refresh_token.clone());
    let expires_in = data.expires_in;

    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64 + (expires_in as i64 * 1000))
        .unwrap_or(0);

    log::info!("OAuth token refresh succeeded");

    Ok(OAuthTokens {
        access_token: Some(access_token),
        refresh_token: Some(new_refresh_token),
        expires_at: Some(expires_at),
        expires_in: Some(expires_in),
        scopes: parse_scopes(data.scope.as_deref()),
        ..OAuthTokens::default()
    })
}

/// Fetch and store user roles from the OAuth token.
pub async fn fetch_and_store_user_roles(_access_token: &str) -> anyhow::Result<()> {
    // This depends on the global config storage which is a separate module.
    // For now, we just return Ok as a placeholder - the actual implementation
    // would update the config with the roles.
    log::debug!("Fetching and storing user roles");
    Ok(())
}

/// Create and store an API key from the OAuth access token.
pub async fn create_and_store_api_key(access_token: &str) -> anyhow::Result<Option<String>> {
    let config = get_oauth_config();

    let client = crate::http_client::new_http_client();
    let response = client
        .post(&config.api_key_url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await?;

    if response.status() != 200 {
        let status_text = response.text().await.unwrap_or_default();
        log::error!("Failed to create API key: {status_text}");
        return Err(anyhow::anyhow!("Failed to create API key: {status_text}"));
    }

    // Parse the response to get the raw key
    let json: serde_json::Value = response.json().await?;
    let api_key = json.get("raw_key").and_then(|v| v.as_str()).map(|s| s.to_string());

    if let Some(ref key) = api_key {
        log::info!("API key created successfully");
    }

    Ok(api_key)
}

/// Check if an OAuth token has expired (with a 5-minute buffer).
pub fn is_oauth_token_expired(expires_at: Option<i64>) -> bool {
    let expires_at = match expires_at {
        Some(ea) => ea,
        None => return false,
    };

    // Buffer of 5 minutes
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let buffer_time = 5 * 60 * 1000; // 5 minutes in ms
    let expires_with_buffer = now + buffer_time;
    expires_with_buffer >= expires_at
}
