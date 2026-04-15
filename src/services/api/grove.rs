// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/grove.ts
//! Grove notification service
//! Handles Grove privacy settings and notifications

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

/// Cache expiration: 24 hours
const GROVE_CACHE_EXPIRATION_MS: u64 = 24 * 60 * 60 * 1000;

/// Account settings from API
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSettings {
    pub grove_enabled: Option<bool>,
    #[serde(rename = "grove_notice_viewed_at")]
    pub grove_notice_viewed_at: Option<String>,
}

/// Grove configuration from Statsig API
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroveConfig {
    pub grove_enabled: bool,
    #[serde(default)]
    pub domain_excluded: bool,
    #[serde(default = "default_true")]
    pub notice_is_grace_period: bool,
    pub notice_reminder_frequency: Option<u32>,
}

fn default_true() -> bool {
    true
}

/// API result type that distinguishes between failure and success
#[derive(Debug, Clone)]
pub enum ApiResult<T> {
    Success { data: T },
    Failure,
}

impl<T> ApiResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, ApiResult::Success { .. })
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            ApiResult::Success { data } => Some(data),
            ApiResult::Failure => None,
        }
    }
}

/// Grove cache entry
#[derive(Debug, Clone)]
pub struct GroveCacheEntry {
    pub grove_enabled: bool,
    pub timestamp: u64,
}

/// Global cache for grove config
static GROVE_CONFIG_CACHE: Lazy<Mutex<HashMap<String, GroveCacheEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get current timestamp in milliseconds
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Check if essential traffic only mode is enabled
fn is_essential_traffic_only() -> bool {
    // Check if we're in essential-only mode (privacy level)
    std::env::var("AI_CODE_PRIVACY_LEVEL")
        .map(|v| v == "essential")
        .unwrap_or(false)
}

/// Check if user is a consumer subscriber
fn is_consumer_subscriber() -> bool {
    // TODO: Integrate with auth system
    false
}

/// Get OAuth account info
fn get_oauth_account_info() -> Option<OAuthAccountInfo> {
    // TODO: Integrate with auth system
    None
}

#[derive(Debug, Clone)]
pub struct OAuthAccountInfo {
    pub account_uuid: String,
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

/// Get auth headers for API calls
fn get_auth_headers() -> Result<reqwest::header::HeaderMap, String> {
    // TODO: Integrate with auth system
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );
    Ok(headers)
}

/// Get user agent
fn get_user_agent() -> String {
    format!("ai-agent/{}", env!("CARGO_PKG_VERSION"))
}

/// Grove settings cache
static GROVE_SETTINGS_CACHE: Lazy<Mutex<Option<AccountSettings>>> =
    Lazy::new(|| Mutex::new(None));

/// Grove notice config cache
static GROVE_NOTICE_CONFIG_CACHE: Lazy<Mutex<Option<GroveConfig>>> =
    Lazy::new(|| Mutex::new(None));

/// Clear grove settings cache
pub fn clear_grove_settings_cache() {
    if let Ok(mut cache) = GROVE_SETTINGS_CACHE.lock() {
        *cache = None;
    }
    // Also clear memoized config
    if let Ok(mut cache) = GROVE_NOTICE_CONFIG_CACHE.lock() {
        *cache = None;
    }
}

/// Get Grove settings for the user account
pub async fn get_grove_settings() -> ApiResult<AccountSettings> {
    // Grove is a notification feature; during an outage, skipping it is correct.
    if is_essential_traffic_only() {
        return ApiResult::Failure;
    }

    // Check cache first
    {
        let cache = GROVE_SETTINGS_CACHE.lock().unwrap();
        if let Some(ref settings) = *cache {
            return ApiResult::Success { data: settings.clone() };
        }
    }

    // Fetch from API
    let result = fetch_grove_settings().await;

    // Cache on success
    if let ApiResult::Success { ref data } = result {
        let mut cache = GROVE_SETTINGS_CACHE.lock().unwrap();
        *cache = Some(data.clone());
    }

    result
}

/// Fetch Grove settings from API
async fn fetch_grove_settings() -> ApiResult<AccountSettings> {
    let config = get_oauth_config();
    let url = format!("{}/api/oauth/account/settings", config.base_api_url);

    let auth_headers = match get_auth_headers() {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to get auth headers: {}", e);
            return ApiResult::Failure;
        }
    };

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .headers(auth_headers)
        .header("User-Agent", get_user_agent())
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<AccountSettings>().await {
                Ok(data) => ApiResult::Success { data },
                Err(e) => {
                    log::error!("Failed to parse grove settings: {}", e);
                    clear_grove_settings_cache();
                    ApiResult::Failure
                }
            }
        }
        Err(e) => {
            log::error!("Failed to fetch grove settings: {}", e);
            clear_grove_settings_cache();
            ApiResult::Failure
        }
    }
}

/// Mark that the Grove notice has been viewed by the user
pub async fn mark_grove_notice_viewed() -> Result<(), ()> {
    let config = get_oauth_config();
    let url = format!("{}/api/oauth/account/grove_notice_viewed", config.base_api_url);

    let auth_headers = match get_auth_headers() {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to get auth headers: {}", e);
            return Err(());
        }
    };

    let client = reqwest::Client::new();
    match client
        .post(&url)
        .headers(auth_headers)
        .header("User-Agent", get_user_agent())
        .send()
        .await
    {
        Ok(_) => {
            // Invalidate cache
            clear_grove_settings_cache();
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to mark grove notice viewed: {}", e);
            Err(())
        }
    }
}

/// Update Grove settings for the user account
pub async fn update_grove_settings(grove_enabled: bool) -> Result<(), ()> {
    let config = get_oauth_config();
    let url = format!("{}/api/oauth/account/settings", config.base_api_url);

    let auth_headers = match get_auth_headers() {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to get auth headers: {}", e);
            return Err(());
        }
    };

    let client = reqwest::Client::new();
    let body = serde_json::json!({ "grove_enabled": grove_enabled });

    match client
        .patch(&url)
        .headers(auth_headers)
        .header("User-Agent", get_user_agent())
        .json(&body)
        .send()
        .await
    {
        Ok(_) => {
            // Invalidate cache
            clear_grove_settings_cache();
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to update grove settings: {}", e);
            Err(())
        }
    }
}

/// Check if user is qualified for Grove (non-blocking, cache-first)
pub async fn is_qualified_for_grove() -> bool {
    if !is_consumer_subscriber() {
        return false;
    }

    let account_id = match get_oauth_account_info() {
        Some(info) => info.account_uuid,
        None => return false,
    };

    // Check cache
    let cached_entry = {
        let cache = GROVE_CONFIG_CACHE.lock().unwrap();
        cache.get(&account_id).cloned()
    };

    let now = now_ms();

    // No cache - trigger background fetch and return false (non-blocking)
    if let None = cached_entry {
        log::debug!(
            "Grove: No cache, fetching config in background (dialog skipped this session)"
        );
        let account_id_clone = account_id.clone();
        tokio::spawn(async move {
            let _ = fetch_and_store_grove_config(&account_id_clone).await;
        });
        return false;
    }

    let entry = cached_entry.unwrap();

    // Cache exists but is stale - return cached value and refresh in background
    if now - entry.timestamp > GROVE_CACHE_EXPIRATION_MS {
        log::debug!(
            "Grove: Cache stale, returning cached data and refreshing in background"
        );
        let account_id_clone = account_id.clone();
        tokio::spawn(async move {
            let _ = fetch_and_store_grove_config(&account_id_clone).await;
        });
        return entry.grove_enabled;
    }

    // Cache is fresh - return it immediately
    log::debug!("Grove: Using fresh cached config");
    entry.grove_enabled
}

/// Fetch Grove config from API and store in cache
async fn fetch_and_store_grove_config(account_id: &str) -> Result<(), ()> {
    let result = get_grove_notice_config().await;

    let grove_enabled = match result {
        ApiResult::Success { data } => data.grove_enabled,
        ApiResult::Failure => return Err(()),
    };

    let should_cache = {
        let cache = GROVE_CONFIG_CACHE.lock().unwrap();
        let cached = cache.get(account_id);
        match cached {
            Some(entry) => {
                // Only cache if value changed or not stale
                entry.grove_enabled != grove_enabled
                    || (now_ms() - entry.timestamp) > GROVE_CACHE_EXPIRATION_MS
            }
            None => true,
        }
    };

    if should_cache {
        let mut cache = GROVE_CONFIG_CACHE.lock().unwrap();
        cache.insert(
            account_id.to_string(),
            GroveCacheEntry {
                grove_enabled,
                timestamp: now_ms(),
            },
        );
    }

    Ok(())
}

/// Get Grove notice config from Statsig API
pub async fn get_grove_notice_config() -> ApiResult<GroveConfig> {
    // Grove is a notification feature; during an outage, skipping it is correct.
    if is_essential_traffic_only() {
        return ApiResult::Failure;
    }

    // Check cache first
    {
        let cache = GROVE_NOTICE_CONFIG_CACHE.lock().unwrap();
        if let Some(ref config) = *cache {
            return ApiResult::Success { data: config.clone() };
        }
    }

    // Fetch from API
    let result = fetch_grove_notice_config().await;

    // Cache on success
    if let ApiResult::Success { ref data } = result {
        let mut cache = GROVE_NOTICE_CONFIG_CACHE.lock().unwrap();
        *cache = Some(data.clone());
    }

    result
}

/// Fetch Grove notice config from API
async fn fetch_grove_notice_config() -> ApiResult<GroveConfig> {
    let config = get_oauth_config();
    let url = format!("{}/api/claude_code_grove", config.base_api_url);

    let auth_headers = match get_auth_headers() {
        Ok(h) => h,
        Err(e) => {
            log::error!("Failed to get auth headers: {}", e);
            return ApiResult::Failure;
        }
    };

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(3000))
        .build()
    {
        Ok(c) => c,
        Err(_) => return ApiResult::Failure,
    };

    match client
        .get(&url)
        .headers(auth_headers)
        .header("User-Agent", get_user_agent())
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<GroveConfig>().await {
                Ok(data) => ApiResult::Success { data },
                Err(e) => {
                    log::debug!("Failed to parse grove notice config: {}", e);
                    ApiResult::Failure
                }
            }
        }
        Err(e) => {
            log::debug!("Failed to fetch grove notice config: {}", e);
            ApiResult::Failure
        }
    }
}

/// Determines whether the Grove dialog should be shown
pub fn calculate_should_show_grove(
    settings_result: &ApiResult<AccountSettings>,
    config_result: &ApiResult<GroveConfig>,
    show_if_already_viewed: bool,
) -> bool {
    // Hide dialog on API failure (after retry)
    let settings = match settings_result {
        ApiResult::Success { data } => data,
        ApiResult::Failure => return false,
    };

    let config = match config_result {
        ApiResult::Success { data } => data,
        ApiResult::Failure => return false,
    };

    let has_chosen = settings.grove_enabled.is_some();
    if has_chosen {
        return false;
    }

    if show_if_already_viewed {
        return true;
    }

    if !config.notice_is_grace_period {
        return true;
    }

    // Check if we need to remind the user
    if let Some(reminder_frequency) = config.notice_reminder_frequency {
        if let Some(viewed_at) = &settings.grove_notice_viewed_at {
            if let Ok(viewed_time) = viewed_at.parse::<i64>() {
                let days_since_viewed = (now_ms() as i64 - viewed_time) / (1000 * 60 * 60 * 24);
                return days_since_viewed >= reminder_frequency as i64;
            }
        }
    }

    // Show if never viewed before
    settings.grove_notice_viewed_at.is_none()
}

/// Check Grove for non-interactive mode
pub async fn check_grove_for_non_interactive() {
    let settings_result = get_grove_settings().await;
    let config_result = get_grove_notice_config().await;

    let should_show_grove = calculate_should_show_grove(
        &settings_result,
        &config_result,
        false,
    );

    if should_show_grove {
        let config = config_result.data();

        // TODO: Implement analytics event
        // logEvent('tengu_grove_print_viewed', {...})

        if config.map(|c| c.notice_is_grace_period).unwrap_or(true) {
            // Grace period is still active - show informational message and continue
            eprintln!(
                "\nAn update to our Consumer Terms and Privacy Policy will take effect on October 8, 2025. Run `claude` to review the updated terms.\n\n"
            );
            let _ = mark_grove_notice_viewed().await;
        } else {
            // Grace period has ended - show error message and exit
            eprintln!(
                "\n[ACTION REQUIRED] An update to our Consumer Terms and Privacy Policy has taken effect on October 8, 2025. You must run `claude` to review the updated terms.\n\n"
            );
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_should_show_grove_failure() {
        let result = calculate_should_show_grove(
            &ApiResult::Failure,
            &ApiResult::Success {
                data: GroveConfig {
                    grove_enabled: true,
                    domain_excluded: false,
                    notice_is_grace_period: true,
                    notice_reminder_frequency: None,
                },
            },
            false,
        );
        assert!(!result);
    }

    #[test]
    fn test_calculate_should_show_grove_already_chosen() {
        let result = calculate_should_show_grove(
            &ApiResult::Success {
                data: AccountSettings {
                    grove_enabled: Some(true),
                    grove_notice_viewed_at: None,
                },
            },
            &ApiResult::Success {
                data: GroveConfig {
                    grove_enabled: true,
                    domain_excluded: false,
                    notice_is_grace_period: true,
                    notice_reminder_frequency: None,
                },
            },
            false,
        );
        assert!(!result);
    }
}