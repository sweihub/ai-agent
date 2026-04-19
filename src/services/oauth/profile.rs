// Source: /data/home/swei/claudecode/openclaudecode/src/services/oauth/getOauthProfile.ts
//! Fetch user profile information from OAuth tokens and API keys.
//!
//! Two endpoints are supported:
//! - Profile from OAuth access token (Bearer auth)
//! - Profile from API key + account UUID (anthropic-beta header)
//!
//! Source: /data/home/swei/claudecode/openclaudecode/src/services/oauth/getOauthProfile.ts

use crate::services::oauth::constants::get_oauth_config;
use crate::services::oauth::types::OAuthProfileResponse;

/// Fetch the OAuth profile using an OAuth access token (Bearer auth).
pub async fn get_oauth_profile_from_oauth_token(
    access_token: &str,
) -> Option<OAuthProfileResponse> {
    let config = get_oauth_config();
    let endpoint = format!("{}/api/oauth/profile", config.base_api_url);

    let client = reqwest::Client::builder()
        .user_agent(crate::utils::user_agent::get_user_agent())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Content-Type", "application/json")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        log::warn!(
            "Failed to fetch OAuth profile: status={}",
            response.status()
        );
        return None;
    }

    response.json().await.ok()
}

/// Fetch the OAuth profile using an API key and account UUID.
///
/// This is used in interactive sessions where the user has an API key
/// stored but needs to link their OAuth account.
pub async fn get_oauth_profile_from_api_key(
    account_uuid: &str,
    api_key: &str,
) -> Option<OAuthProfileResponse> {
    let config = get_oauth_config();
    let endpoint = format!("{}/api/claude_cli_profile", config.base_api_url);

    let client = reqwest::Client::builder()
        .user_agent(crate::utils::user_agent::get_user_agent())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client
        .get(&endpoint)
        .header("x-api-key", api_key)
        .header("anthropic-beta", crate::services::oauth::types::OAUTH_BETA_HEADER)
        .query(&[("account_uuid", account_uuid)])
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        log::warn!(
            "Failed to fetch OAuth profile from API key: status={}",
            response.status()
        );
        return None;
    }

    response.json().await.ok()
}

/// Extract subscription type from the organization type in the profile.
pub fn extract_subscription_type(profile: &OAuthProfileResponse) -> Option<String> {
    let org_type = profile
        .extra
        .get("organization")
        .and_then(|v| v.get("organization_type"))
        .and_then(|v| v.as_str())?;

    match org_type {
        "claude_max" => Some("max".to_string()),
        "claude_pro" => Some("pro".to_string()),
        "claude_enterprise" => Some("enterprise".to_string()),
        "claude_team" => Some("team".to_string()),
        _ => None,
    }
}

/// Extract the organization UUID from the profile.
pub fn extract_organization_uuid(profile: &OAuthProfileResponse) -> Option<String> {
    profile
        .extra
        .get("organization")
        .and_then(|v| v.get("uuid"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Extract the rate limit tier from the profile.
pub fn extract_rate_limit_tier(profile: &OAuthProfileResponse) -> Option<String> {
    profile
        .extra
        .get("organization")
        .and_then(|v| v.get("rate_limit_tier"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}
