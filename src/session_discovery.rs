// Source: /data/home/swei/claudecode/openclaudecode/src/assistant/sessionDiscovery.ts
//! Session discovery for assistant sessions - discover sessions from the remote API

use crate::constants::env::{ai, ai_code};
use crate::utils::http::get_user_agent;
use serde::{Deserialize, Serialize};

/// Assistant session discovered from the remote API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantSession {
    pub id: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub model: Option<String>,
    pub summary: Option<String>,
    pub tag: Option<String>,
    #[serde(rename = "messageCount")]
    pub message_count: Option<u32>,
    #[serde(rename = "cwd")]
    pub working_directory: Option<String>,
}

/// Discovery result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub sessions: Vec<AssistantSession>,
    pub success: bool,
    pub error: Option<String>,
    #[serde(rename = "totalCount")]
    pub total_count: Option<u32>,
}

/// Get the base URL for the session discovery API
fn get_discovery_api_base_url() -> String {
    // Check for custom API base URL
    if let Ok(base_url) = std::env::var(ai::API_BASE_URL) {
        return base_url.trim_end_matches('/').to_string();
    }
    if let Ok(base_url) = std::env::var(ai::BASE_URL) {
        return base_url.trim_end_matches('/').to_string();
    }
    "https://api.anthropic.com".to_string()
}

/// Get the OAuth token for authentication
fn get_oauth_token() -> Option<String> {
    // Check various environment variables for the token
    std::env::var(ai_code::OAUTH_TOKEN)
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| {
            std::env::var(ai::OAUTH_TOKEN)
                .ok()
                .filter(|t| !t.is_empty())
        })
        .or_else(|| {
            std::env::var(ai::AUTH_TOKEN)
                .ok()
                .filter(|t| !t.is_empty())
        })
        .or_else(|| {
            std::env::var(ai::API_KEY)
                .ok()
                .filter(|t| !t.is_empty())
        })
}

/// Build HTTP headers for the discovery request
fn build_discovery_headers() -> Result<reqwest::header::HeaderMap, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "anthropic-version",
        reqwest::header::HeaderValue::from_static("2025-04-20"),
    );

    if let Some(token) = get_oauth_token() {
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&auth_value)
                .map_err(|e| format!("Invalid auth header: {}", e))?,
        );
    }

    headers.insert(
        "User-Agent",
        reqwest::header::HeaderValue::from_str(&get_user_agent())
            .map_err(|e| format!("Invalid User-Agent header: {}", e))?,
    );

    Ok(headers)
}

/// Discover assistant sessions from the remote API
pub async fn discover_assistant_sessions() -> Vec<serde_json::Value> {
    let base_url = get_discovery_api_base_url();
    let url = format!("{}/api/claude_code/sessions", base_url);

    let headers = match build_discovery_headers() {
        Ok(h) => h,
        Err(e) => {
            log::warn!("Failed to build discovery headers: {}", e);
            return Vec::new();
        }
    };

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to build HTTP client: {}", e);
            return Vec::new();
        }
    };

    let response = match client.get(&url).headers(headers).send().await {
        Ok(r) => r,
        Err(e) => {
            log::debug!("Session discovery request failed: {}", e);
            return Vec::new();
        }
    };

    if !response.status().is_success() {
        log::debug!(
            "Session discovery returned non-success status: {}",
            response.status()
        );
        return Vec::new();
    }

    match response.json::<serde_json::Value>().await {
        Ok(json) => {
            // Extract sessions array from response
            if let Some(sessions) = json.get("sessions").and_then(|s| s.as_array()) {
                sessions.clone()
            } else if json.is_array() {
                json.as_array().cloned().unwrap_or_default()
            } else {
                Vec::new()
            }
        }
        Err(e) => {
            log::debug!("Failed to parse discovery response: {}", e);
            Vec::new()
        }
    }
}

/// Discover sessions with structured result
pub async fn discover_sessions_with_result() -> DiscoveryResult {
    let base_url = get_discovery_api_base_url();
    let url = format!("{}/api/claude_code/sessions", base_url);

    let headers = match build_discovery_headers() {
        Ok(h) => h,
        Err(e) => {
            return DiscoveryResult {
                sessions: Vec::new(),
                success: false,
                error: Some(format!("Failed to build headers: {}", e)),
                total_count: None,
            };
        }
    };

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return DiscoveryResult {
                sessions: Vec::new(),
                success: false,
                error: Some(format!("Failed to build client: {}", e)),
                total_count: None,
            };
        }
    };

    let response = match client.get(&url).headers(headers).send().await {
        Ok(r) => r,
        Err(e) => {
            return DiscoveryResult {
                sessions: Vec::new(),
                success: false,
                error: Some(format!("Request failed: {}", e)),
                total_count: None,
            };
        }
    };

    if !response.status().is_success() {
        let status = response.status().as_u16();
        return DiscoveryResult {
            sessions: Vec::new(),
            success: false,
            error: Some(format!("API returned status: {}", status)),
            total_count: None,
        };
    }

    match response.json::<serde_json::Value>().await {
        Ok(json) => {
            let sessions = if let Some(sessions_arr) = json.get("sessions").and_then(|s| s.as_array()) {
                sessions_arr
                    .iter()
                    .filter_map(|s| serde_json::from_value::<AssistantSession>(s.clone()).ok())
                    .collect()
            } else {
                Vec::new()
            };

            let total_count = json
                .get("totalCount")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32);

            DiscoveryResult {
                sessions,
                success: true,
                error: None,
                total_count,
            }
        }
        Err(e) => DiscoveryResult {
            sessions: Vec::new(),
            success: false,
            error: Some(format!("Failed to parse response: {}", e)),
            total_count: None,
        },
    }
}

/// Check if session discovery is available (has auth credentials)
pub fn is_discovery_available() -> bool {
    get_oauth_token().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_discovery_available_no_token() {
        // By default, no token should be set in test environment
        let available = is_discovery_available();
        assert!(!available);
    }

    #[test]
    fn test_get_api_base_url_default() {
        // Without env vars set, should return default
        let url = get_discovery_api_base_url();
        assert_eq!(url, "https://api.anthropic.com");
    }
}
