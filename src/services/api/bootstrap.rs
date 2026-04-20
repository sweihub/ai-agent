// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/bootstrap.ts
//! Bootstrap module
//! Fetches bootstrap data from the API

use crate::utils::user_agent::get_user_agent;
use std::collections::HashMap;

use crate::constants::oauth::OAUTH_BETA_HEADER;
use crate::constants::oauth::get_oauth_config as get_oauth_config_impl;

/// Bootstrap response from API
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct BootstrapResponse {
    #[serde(default)]
    pub client_data: Option<serde_json::Value>,
    #[serde(default)]
    pub additional_model_options: Option<Vec<AdditionalModelOption>>,
}

/// Additional model option
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct AdditionalModelOption {
    pub model: String,
    pub name: String,
    pub description: String,
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("[Bootstrap] {}", msg);
}

/// Check if essential traffic only mode is enabled
fn is_essential_traffic_only() -> bool {
    std::env::var("AI_CODE_PRIVACY_LEVEL")
        .map(|v| v == "essential")
        .unwrap_or(false)
}

/// Get API provider
fn get_api_provider() -> String {
    std::env::var("AI_CODE_PROVIDER").unwrap_or_else(|_| "firstParty".to_string())
}

/// Get OAuth config (simplified version matching TypeScript)
fn get_oauth_config() -> OauthConfig {
    OauthConfig {
        base_api_url: std::env::var("AI_CODE_API_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct OauthConfig {
    pub base_api_url: String,
}

/// Get OAuth beta header
fn get_oauth_beta_header() -> String {
    OAUTH_BETA_HEADER.to_string()
}

/// Get Anthropic API key
fn get_anthropic_api_key() -> Option<String> {
    // Try ANTHROPIC_API_KEY first (legacy), then AI_API_KEY
    std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .or_else(|| std::env::var("AI_API_KEY").ok())
}

/// OAuth tokens structure
#[derive(Debug, Clone)]
pub struct OAuthTokens {
    pub access_token: Option<String>,
}

/// Get Claude AI OAuth tokens
fn get_claude_ai_oauth_tokens() -> Option<OAuthTokens> {
    // Check for force-set OAuth token from environment variable
    if let Ok(token) = std::env::var("AI_CODE_OAUTH_TOKEN") {
        return Some(OAuthTokens {
            access_token: Some(token),
        });
    }
    // TODO: Integrate with full auth system (keychain, file descriptor, etc.)
    None
}

/// Check if user has profile scope (simplified - assumes OAuth tokens have profile scope)
fn has_profile_scope() -> bool {
    // If we have OAuth tokens, assume we have profile scope
    get_claude_ai_oauth_tokens().is_some()
}

/// Get global config
fn get_global_config() -> GlobalConfig {
    // TODO: Integrate with real config system
    GlobalConfig::default()
}

#[derive(Debug, Clone, Default)]
pub struct GlobalConfig {
    pub client_data_cache: Option<serde_json::Value>,
    pub additional_model_options_cache: Option<Vec<AdditionalModelOption>>,
}

/// Save global config
fn save_global_config(_update: impl FnOnce(&mut GlobalConfig)) {
    // TODO: Integrate with real config system
}

/// Get user agent (delegates to unified function).
fn get_claude_code_user_agent() -> String {
    get_user_agent()
}

/// Fetch bootstrap data from API
async fn fetch_bootstrap_api() -> Option<BootstrapResponse> {
    if is_essential_traffic_only() {
        log_for_debugging("Skipped: Nonessential traffic disabled");
        return None;
    }

    if get_api_provider() != "firstParty" {
        log_for_debugging("Skipped: 3P provider");
        return None;
    }

    // OAuth preferred (requires user:profile scope — service-key OAuth tokens
    // lack it and would 403). Fall back to API key auth for console users.
    let api_key = get_anthropic_api_key();
    let has_usable_oauth = get_claude_ai_oauth_tokens()
        .map(|t| t.access_token.as_ref().map(|s| !s.is_empty()).unwrap_or(false))
        .unwrap_or(false)
        && has_profile_scope();

    if !has_usable_oauth && api_key.is_none() {
        log_for_debugging("Skipped: no usable OAuth or API key");
        return None;
    }

    let config = get_oauth_config_impl();
    let endpoint = format!("{}/api/claude_cli/bootstrap", config.base_api_url);

    log_for_debugging("Fetching");

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(5000))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log_for_debugging(&format!("Failed to build client: {}", e));
            return None;
        }
    };

    // Build auth headers
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("User-Agent".to_string(), get_claude_code_user_agent());

    if let Some(token) = get_claude_ai_oauth_tokens() {
        if let Some(access_token) = token.access_token {
            if has_profile_scope() {
                headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));
                headers.insert("anthropic-beta".to_string(), get_oauth_beta_header());
            }
        }
    }

    if headers.get("Authorization").is_none() {
        if let Some(key) = &api_key {
            headers.insert("x-api-key".to_string(), key.clone());
        }
    }

    let reqwest_headers: reqwest::header::HeaderMap = headers
        .into_iter()
        .filter_map(|(k, v)| {
            let key: reqwest::header::HeaderName = k.parse().ok()?;
            let value: reqwest::header::HeaderValue = v.parse().ok()?;
            Some((key, value))
        })
        .collect();

    let response = match client.get(&endpoint).headers(reqwest_headers).send().await {
        Ok(resp) => resp,
        Err(e) => {
            log_for_debugging(&format!("Fetch failed: {}", e));
            return None;
        }
    };

    if !response.status().is_success() {
        log_for_debugging(&format!("Fetch failed: status {}", response.status()));
        return None;
    }

    match response.json::<BootstrapResponse>().await {
        Ok(data) => {
            log_for_debugging("Fetch ok");
            Some(data)
        }
        Err(e) => {
            log_for_debugging(&format!("Response failed validation: {}", e));
            None
        }
    }
}

/// Fetch bootstrap data from the API and persist to disk cache.
pub async fn fetch_bootstrap_data() {
    let response = match fetch_bootstrap_api().await {
        Some(r) => r,
        None => return,
    };

    let client_data = response.client_data.unwrap_or(serde_json::Value::Null);
    let additional_model_options = response.additional_model_options.unwrap_or_default();

    // Only persist if data actually changed — avoids a config write on every startup.
    let config = get_global_config();

    let client_data_unchanged = config.client_data_cache.as_ref() == Some(&client_data);
    let model_options_unchanged = config.additional_model_options_cache.as_ref()
        .map(|c| c == &additional_model_options)
        .unwrap_or(false);

    if client_data_unchanged && model_options_unchanged {
        log_for_debugging("Cache unchanged, skipping write");
        return;
    }

    log_for_debugging("Cache updated, persisting to disk");
    save_global_config(|cfg| {
        cfg.client_data_cache = Some(client_data);
        cfg.additional_model_options_cache = Some(additional_model_options);
    });
}