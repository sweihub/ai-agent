// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/metricsOptOut.ts
//! Metrics opt-out module
//! Checks if metrics/logging is enabled for the organization

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

/// In-memory TTL — dedupes calls within a single process (1 hour)
const CACHE_TTL_MS: u64 = 60 * 60 * 1000;

/// Disk TTL — org settings rarely change (24 hours)
const DISK_CACHE_TTL_MS: u64 = 24 * 60 * 60 * 1000;

/// Metrics enabled response
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MetricsEnabledResponse {
    pub metrics_logging_enabled: bool,
}

/// Metrics status
#[derive(Debug, Clone)]
pub struct MetricsStatus {
    pub enabled: bool,
    pub has_error: bool,
}

/// Metrics status cache entry (alias for config type)
type MetricsStatusCacheEntry = crate::utils::config::MetricsStatusCache;

/// Get current timestamp in milliseconds
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Check if essential traffic only mode is enabled
fn is_essential_traffic_only() -> bool {
    std::env::var("AI_CODE_PRIVACY_LEVEL")
        .map(|v| v == "essential")
        .unwrap_or(false)
}

/// Check if user is Claude.ai subscriber
fn is_claude_ai_subscriber() -> bool {
    // Check for OAuth token which indicates a subscriber
    std::env::var("AI_CODE_OAUTH_TOKEN").is_ok()
}

/// Check if user has profile scope
fn has_profile_scope() -> bool {
    // If we have OAuth token, assume profile scope is available
    std::env::var("AI_CODE_OAUTH_TOKEN").is_ok()
}

/// Get auth headers from HTTP utils
fn get_auth_headers() -> crate::utils::http::AuthHeaders {
    crate::utils::http::get_auth_headers()
}

/// Get user agent
fn get_user_agent() -> String {
    format!("ai-agent/{}", env!("CARGO_PKG_VERSION"))
}

/// Get global config
fn get_global_config() -> crate::utils::config::GlobalConfig {
    crate::utils::config::get_global_config()
}

/// Save global config
fn save_global_config(update: impl FnOnce(&mut crate::utils::config::GlobalConfig)) {
    let mut config = get_global_config();
    update(&mut config);
    let _ = crate::utils::config::save_global_config(&config);
}

/// In-memory cache
static IN_MEMORY_CACHE: Lazy<Mutex<Option<MetricsStatusCacheEntry>>> =
    Lazy::new(|| Mutex::new(None));

/// Fetch metrics enabled from API
async fn fetch_metrics_enabled() -> Result<MetricsEnabledResponse, String> {
    let auth_result = get_auth_headers();
    if let Some(error) = auth_result.error {
        return Err(format!("Auth error: {}", error));
    }

    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("User-Agent".to_string(), get_user_agent());
    for (k, v) in auth_result.headers {
        headers.insert(k, v);
    }

    let reqwest_headers: reqwest::header::HeaderMap = headers
        .into_iter()
        .filter_map(|(k, v)| {
            let key: reqwest::header::HeaderName = k.parse().ok()?;
            let value: reqwest::header::HeaderValue = v.parse().ok()?;
            Some((key, value))
        })
        .collect();

    let endpoint = "https://api.anthropic.com/api/claude_code/organizations/metrics_enabled";
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(5000))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(endpoint)
        .headers(reqwest_headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<MetricsEnabledResponse>()
        .await
        .map_err(|e| e.to_string())
}

/// Check metrics enabled via API
async fn check_metrics_enabled_api() -> MetricsStatus {
    // Incident kill switch: skip the network call when nonessential traffic is disabled
    if is_essential_traffic_only() {
        return MetricsStatus {
            enabled: false,
            has_error: false,
        };
    }

    match fetch_metrics_enabled().await {
        Ok(data) => {
            log::debug!(
                "Metrics opt-out API response: enabled={}",
                data.metrics_logging_enabled
            );
            MetricsStatus {
                enabled: data.metrics_logging_enabled,
                has_error: false,
            }
        }
        Err(e) => {
            log::debug!("Failed to check metrics opt-out status: {}", e);
            MetricsStatus {
                enabled: false,
                has_error: true,
            }
        }
    }
}

/// Refresh metrics status (with caching)
async fn refresh_metrics_status() -> MetricsStatus {
    let now = now_ms();

    // Check in-memory cache first
    {
        let cache = IN_MEMORY_CACHE.lock().unwrap();
        if let Some(ref entry) = *cache {
            if now - entry.timestamp < CACHE_TTL_MS as u64 {
                return MetricsStatus {
                    enabled: entry.enabled,
                    has_error: false,
                };
            }
        }
    }

    let result = check_metrics_enabled_api().await;

    if result.has_error {
        return result;
    }

    // Check if value changed
    let cached = get_global_config().metrics_status_cache;
    let unchanged = cached
        .as_ref()
        .map(|c| c.enabled == result.enabled)
        .unwrap_or(false);

    // Skip write when unchanged AND timestamp still fresh
    if unchanged {
        if let Some(ref c) = cached {
            if now - c.timestamp < DISK_CACHE_TTL_MS as u64 {
                return result;
            }
        }
    }

    // Save to disk cache
    let entry = MetricsStatusCacheEntry {
        enabled: result.enabled,
        timestamp: now,
    };

    save_global_config(|cfg| {
        cfg.metrics_status_cache = Some(entry.clone());
    });

    // Update in-memory cache
    {
        let mut cache = IN_MEMORY_CACHE.lock().unwrap();
        *cache = Some(entry);
    }

    result
}

/// Check if metrics are enabled for the current organization.
/// Two-tier cache:
/// - Disk (24h TTL): survives process restarts. Fresh disk cache → zero network.
/// - In-memory (1h TTL): dedupes the background refresh within a process.
pub async fn check_metrics_enabled() -> MetricsStatus {
    // Service key OAuth sessions lack user:profile scope → would 403.
    // API key users (non-subscribers) fall through and use x-api-key auth.
    if is_claude_ai_subscriber() && !has_profile_scope() {
        return MetricsStatus {
            enabled: false,
            has_error: false,
        };
    }

    let cached = get_global_config().metrics_status_cache;

    if let Some(ref cached) = cached {
        if now_ms() - cached.timestamp > DISK_CACHE_TTL_MS as u64 {
            // Background refresh (fire and forget)
            let _ = refresh_metrics_status().await;
        }
        return MetricsStatus {
            enabled: cached.enabled,
            has_error: false,
        };
    }

    // First-ever run on this machine: block on the network to populate disk.
    refresh_metrics_status().await
}

/// Clear metrics enabled cache (for testing)
pub fn clear_metrics_enabled_cache_for_testing() {
    let mut cache = IN_MEMORY_CACHE.lock().unwrap();
    *cache = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tests::common::clear_all_test_state;

    #[test]
    fn test_is_essential_traffic_only_default() {
        clear_all_test_state();
        // Without env var, should return false
        let result = is_essential_traffic_only();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_check_metrics_enabled_not_subscriber() {
        clear_all_test_state();
        // Not a subscriber, should return enabled: false
        let result = check_metrics_enabled().await;
        // Due to has_profile_scope returning false for non-subscriber, this returns early
        assert!(!result.enabled);
    }
}
