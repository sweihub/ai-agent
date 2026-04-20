// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/ultrareviewQuota.ts
//! Ultrareview quota module
//! Fetches ultrareview quota for subscribers

use std::collections::HashMap;

use crate::utils::http::get_user_agent;

/// Ultrareview quota response
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UltrareviewQuotaResponse {
    pub reviews_used: u32,
    pub reviews_limit: u32,
    pub reviews_remaining: u32,
    pub is_overage: bool,
}

/// Check if user is Claude.ai subscriber
fn is_claude_ai_subscriber() -> bool {
    // TODO: Integrate with auth system
    false
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

/// Get OAuth headers
fn get_oauth_headers(access_token: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));
    headers.insert("User-Agent".to_string(), get_user_agent());
    headers
}

/// Prepare API request
async fn prepare_api_request() -> PrepareApiResult {
    // TODO: Implement properly
    PrepareApiResult {
        access_token: String::new(),
        org_uuid: String::new(),
    }
}

#[derive(Debug, Clone)]
pub struct PrepareApiResult {
    pub access_token: String,
    pub org_uuid: String,
}

/// Peek the ultrareview quota for display and nudge decisions.
/// Consume happens server-side at session creation.
/// Returns null when not a subscriber or the endpoint errors.
pub async fn fetch_ultrareview_quota() -> Option<UltrareviewQuotaResponse> {
    if !is_claude_ai_subscriber() {
        return None;
    }

    let request = prepare_api_request().await;

    let config = get_oauth_config();
    let url = format!("{}/v1/ultrareview/quota", config.base_api_url);

    let mut headers = get_oauth_headers(&request.access_token);
    headers.insert("x-organization-uuid".to_string(), request.org_uuid);

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(5000))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::debug!("fetchUltrareviewQuota failed: {}", e);
            return None;
        }
    };

    let response = client
        .get(&url)
        .headers(
            headers
                .into_iter()
                .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                .collect(),
        )
        .send()
        .await;

    match response {
        Ok(resp) => {
            match resp.json::<UltrareviewQuotaResponse>().await {
                Ok(data) => Some(data),
                Err(e) => {
                    log::debug!("fetchUltrareviewQuota failed: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            log::debug!("fetchUltrareviewQuota failed: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_ultrareview_quota_not_subscriber() {
        // Not a subscriber, should return None
        let result = fetch_ultrareview_quota().await;
        assert!(result.is_none());
    }
}