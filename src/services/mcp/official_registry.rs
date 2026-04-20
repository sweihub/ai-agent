// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/officialRegistry.ts
//! Official MCP registry - checks if an MCP server URL is official

use std::collections::HashSet;
use std::sync::Mutex;

use crate::utils::http::get_user_agent;

/// Registry server entry
#[derive(Debug, serde::Deserialize)]
pub struct RegistryServer {
    #[serde(default)]
    pub remotes: Option<Vec<RemoteUrl>>,
}

/// Remote URL entry
#[derive(Debug, serde::Deserialize)]
pub struct RemoteUrl {
    pub url: String,
}

/// Registry response
#[derive(Debug, serde::Deserialize)]
pub struct RegistryResponse {
    pub servers: Vec<RegistryServer>,
}

/// Normalize URL - strip query string and trailing slash
fn normalize_url(url: &str) -> Option<String> {
    match url::Url::parse(url) {
        Ok(mut u) => {
            u.set_query(None);
            let mut s = u.to_string();
            if s.ends_with('/') {
                s.pop();
            }
            Some(s)
        }
        Err(_) => None,
    }
}

/// Global set of official MCP URLs
static OFFICIAL_URLS: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Fire-and-forget fetch of the official MCP registry.
/// Populates officialUrls for isOfficialMcpUrl lookups.
pub async fn prefetch_official_mcp_urls() {
    // Check if non-essential traffic is disabled
    if std::env::var("AI_CODE_DISABLE_NONESSENTIAL_TRAFFIC").is_ok() {
        return;
    }

    // Fetch from official registry
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();

    let Ok(client) = client else {
        log::debug!("Failed to build HTTP client for MCP registry");
        return;
    };

    let result = client
        .get("https://api.anthropic.com/mcp-registry/v0/servers?version=latest&visibility=commercial")
        .header("Accept", "application/json")
        .header("User-Agent", get_user_agent())
        .send()
        .await;

    match result {
        Ok(response) => {
            if let Ok(data) = response.json::<RegistryResponse>().await {
                let mut urls = HashSet::new();
                for entry in &data.servers {
                    if let Some(remotes) = &entry.remotes {
                        for remote in remotes {
                            if let Some(normalized) = normalize_url(&remote.url) {
                                urls.insert(normalized);
                            }
                        }
                    }
                }

                let mut guard = OFFICIAL_URLS.lock().unwrap();
                *guard = Some(urls);
                log::debug!("[mcp-registry] Loaded official MCP URLs");
            }
        }
        Err(e) => {
            log::debug!("Failed to fetch MCP registry: {}", e);
        }
    }
}

/// Returns true if the given (already-normalized) URL is in the official MCP registry.
/// Undefined registry -> false (fail-closed).
pub fn is_official_mcp_url(normalized_url: &str) -> bool {
    let guard = OFFICIAL_URLS.lock().unwrap();
    guard.as_ref().map_or(false, |urls| urls.contains(normalized_url))
}

/// Reset official URLs for testing
pub fn reset_official_mcp_urls_for_testing() {
    let mut guard = OFFICIAL_URLS.lock().unwrap();
    *guard = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            normalize_url("https://example.com/mcp?foo=bar"),
            Some("https://example.com/mcp".to_string())
        );
        assert_eq!(
            normalize_url("https://example.com/mcp/"),
            Some("https://example.com/mcp".to_string())
        );
    }

    #[test]
   fn test_is_official_mcp_url_empty() {
        // Empty registry returns false
        assert!(!is_official_mcp_url("https://example.com"));
    }
}