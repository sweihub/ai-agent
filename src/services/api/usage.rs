// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/usage.ts
//! Usage API - fetches utilization data from the API

use std::collections::HashMap;

use crate::utils::http::get_user_agent;

/// Rate limit information from the API
#[derive(Debug, Clone, PartialEq, serde::Deserialize, Default)]
pub struct RateLimit {
    /// A percentage from 0 to 100
    pub utilization: Option<f64>,
    /// ISO 8601 timestamp
    #[serde(rename = "resets_at")]
    pub resets_at: Option<String>,
}

/// Extra usage information
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct ExtraUsage {
    #[serde(rename = "is_enabled")]
    pub is_enabled: bool,
    #[serde(rename = "monthly_limit")]
    pub monthly_limit: Option<i64>,
    #[serde(rename = "used_credits")]
    pub used_credits: Option<i64>,
    pub utilization: Option<f64>,
}

/// Utilization data from the API
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct Utilization {
    #[serde(rename = "five_hour")]
    pub five_hour: Option<RateLimit>,
    #[serde(rename = "seven_day")]
    pub seven_day: Option<RateLimit>,
    #[serde(rename = "seven_day_oauth_apps")]
    pub seven_day_oauth_apps: Option<RateLimit>,
    #[serde(rename = "seven_day_opus")]
    pub seven_day_opus: Option<RateLimit>,
    #[serde(rename = "seven_day_sonnet")]
    pub seven_day_sonnet: Option<RateLimit>,
    #[serde(rename = "extra_usage")]
    pub extra_usage: Option<ExtraUsage>,
}

/// Check if user is Claude.ai subscriber
fn is_claude_ai_subscriber() -> bool {
    // Check for OAuth token from env var which indicates a subscriber
    std::env::var("AI_CODE_OAUTH_TOKEN").is_ok()
}

/// Check if user has profile scope
fn has_profile_scope() -> bool {
    // If we have OAuth token, assume profile scope is available
    std::env::var("AI_CODE_OAUTH_TOKEN").is_ok()
}

/// Get OAuth config
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

/// Get auth headers
fn get_auth_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
    headers.insert("User-Agent".to_string(), get_user_agent());
    // Add OAuth token if available
    if let Ok(token) = std::env::var("AI_CODE_OAUTH_TOKEN") {
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
    }
    headers
}

/// Check if OAuth token is expired
fn is_oauth_token_expired(expires_at: &str) -> bool {
    if expires_at.is_empty() {
        return false;
    }
    // Simple check: if expires_at is in the past
    // For now, just return false to allow the request to proceed
    // The API will return 401 if the token is actually expired
    false
}

/// Get OAuth token from environment
fn get_oauth_token() -> Option<String> {
    std::env::var("AI_CODE_OAUTH_TOKEN").ok()
}

/// Fetch utilization data from the API
/// Returns null if not a Claude AI subscriber or no profile scope
pub async fn fetch_utilization() -> Result<Utilization, String> {
    if !is_claude_ai_subscriber() || !has_profile_scope() {
        return Ok(Utilization::default());
    }

    // Skip API call if OAuth token is expired to avoid 401 errors
    if let Some(token) = get_oauth_token() {
        // For now, assume token doesn't expire for simplicity
        // TODO: Integrate proper token expiry check
    }

    let config = get_oauth_config();
    let endpoint = format!("{}/api/oauth/organizations/me/usage", config.base_api_url);

    let headers = get_auth_headers();
    let reqwest_headers: reqwest::header::HeaderMap = headers
        .into_iter()
        .filter_map(|(k, v)| {
            let key: reqwest::header::HeaderName = k.parse().ok()?;
            let value: reqwest::header::HeaderValue = v.parse().ok()?;
            Some((key, value))
        })
        .collect();

    let client = reqwest::Client::new();
    let response = client
        .get(&endpoint)
        .headers(reqwest_headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok(Utilization::default());
    }

    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()));
    }

    response
        .json::<Utilization>()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_default() {
        let rate_limit = RateLimit::default();
        assert_eq!(rate_limit.utilization, None);
        assert_eq!(rate_limit.resets_at, None);
    }

    #[test]
    fn test_extra_usage_default() {
        let extra_usage = ExtraUsage::default();
        assert!(!extra_usage.is_enabled);
        assert_eq!(extra_usage.monthly_limit, None);
    }

    #[test]
    fn test_utilization_default() {
        let utilization = Utilization::default();
        assert_eq!(utilization.five_hour, None);
        assert_eq!(utilization.seven_day, None);
    }
}
