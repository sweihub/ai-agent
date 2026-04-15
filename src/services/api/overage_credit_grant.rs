// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/overageCreditGrant.ts
//! Overage credit grant module
//! Fetches and caches overage credit grant eligibility for subscribers

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

/// Cache TTL - 1 hour
const CACHE_TTL_MS: u64 = 60 * 60 * 1000;

/// Overage credit grant info
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OverageCreditGrantInfo {
    pub available: bool,
    pub eligible: bool,
    pub granted: bool,
    pub amount_minor_units: Option<i64>,
    pub currency: Option<String>,
}

/// Cached grant entry
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CachedGrantEntry {
    pub info: OverageCreditGrantInfo,
    pub timestamp: u64,
}

/// Get current timestamp in milliseconds
fn now_ms() -> u64 {
    SystemTime::now()
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

/// Get OAuth account info
fn get_oauth_account_info() -> Option<OauthAccountInfo> {
    // TODO: Implement properly
    None
}

#[derive(Debug, Clone)]
pub struct OauthAccountInfo {
    pub organization_uuid: String,
}

/// Get global config
fn get_global_config() -> GlobalConfig {
    // TODO: Implement properly
    GlobalConfig::default()
}

#[derive(Debug, Clone, Default)]
pub struct GlobalConfig {
    pub overage_credit_grant_cache: Option<HashMap<String, CachedGrantEntry>>,
}

/// Save global config
fn save_global_config(_update: impl FnOnce(&mut GlobalConfig)) {
    // TODO: Implement properly
}

/// Fetch the current user's overage credit grant eligibility from the backend.
/// The backend resolves tier-specific amounts and role-based claim permission,
/// so the CLI just reads the response without replicating that logic.
async fn fetch_overage_credit_grant() -> Option<OverageCreditGrantInfo> {
    let request = prepare_api_request().await;

    let config = get_oauth_config();
    let url = format!(
        "{}/api/oauth/organizations/{}/overage_credit_grant",
        config.base_api_url, request.org_uuid
    );

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(5000))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::debug!("fetchOverageCreditGrant failed: {}", e);
            return None;
        }
    };

    let headers = get_oauth_headers(&request.access_token);
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
            match resp.json::<OverageCreditGrantInfo>().await {
                Ok(data) => Some(data),
                Err(e) => {
                    log::debug!("fetchOverageCreditGrant failed: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            log::debug!("fetchOverageCreditGrant failed: {}", e);
            None
        }
    }
}

/// Get cached grant info. Returns null if no cache or cache is stale.
/// Callers should render nothing (not block) when this returns null —
/// refreshOverageCreditGrantCache fires lazily to populate it.
pub fn get_cached_overage_credit_grant() -> Option<OverageCreditGrantInfo> {
    let org_id = get_oauth_account_info()?.organization_uuid;
    let config = get_global_config();
    let cache = config.overage_credit_grant_cache?;
    let cached = cache.get(&org_id)?;

    if now_ms() - cached.timestamp > CACHE_TTL_MS {
        return None;
    }

    Some(cached.info.clone())
}

/// Drop the current org's cached entry so the next read refetches.
/// Leaves other orgs' entries intact.
pub fn invalidate_overage_credit_grant_cache() {
    let org_id = match get_oauth_account_info() {
        Some(info) => info.organization_uuid,
        None => return,
    };

    let mut config = get_global_config();
    let cache = match &mut config.overage_credit_grant_cache {
        Some(c) => c,
        None => return,
    };

    if !cache.contains_key(&org_id) {
        return;
    }

    save_global_config(|cfg| {
        if let Some(ref mut c) = cfg.overage_credit_grant_cache {
            c.remove(&org_id);
        }
    });
}

/// Fetch and cache grant info. Fire-and-forget; call when an upsell surface
/// is about to render and the cache is empty.
pub async fn refresh_overage_credit_grant_cache() {
    if is_essential_traffic_only() {
        return;
    }

    let org_id = match get_oauth_account_info() {
        Some(info) => info.organization_uuid,
        None => return,
    };

    let info = match fetch_overage_credit_grant().await {
        Some(i) => i,
        None => return,
    };

    // Skip rewriting info if grant data is unchanged — avoids config write
    // amplification (inc-4552 pattern). Still refresh the timestamp so the
    // TTL-based staleness check in getCachedOverageCreditGrant doesn't keep
    // re-triggering API calls on every component mount.
    save_global_config(|prev| {
        // Derive from prev (lock-fresh) rather than a pre-lock getGlobalConfig()
        // read — saveConfigWithLock re-reads config from disk under the file lock,
        // so another CLI instance may have written between any outer read and lock
        // acquire.
        let prev_cached = prev.overage_credit_grant_cache
            .as_ref()
            .and_then(|c| c.get(&org_id))
            .cloned();
        let existing = prev_cached.as_ref().map(|c| &c.info);

        let data_unchanged = existing.map(|e| {
            e.available == info.available
                && e.eligible == info.eligible
                && e.granted == info.granted
                && e.amount_minor_units == info.amount_minor_units
                && e.currency == info.currency
        }).unwrap_or(false);

        // When data is unchanged and timestamp is still fresh, skip the write entirely
        if data_unchanged {
            if let Some(ref prev_cached) = prev_cached {
                if now_ms() - prev_cached.timestamp <= CACHE_TTL_MS {
                    return;
                }
            }
        }

        let entry = CachedGrantEntry {
            info: if data_unchanged {
                existing.cloned().unwrap_or(info)
            } else {
                info
            },
            timestamp: now_ms(),
        };

        let mut cache = prev.overage_credit_grant_cache.clone().unwrap_or_default();
        cache.insert(org_id, entry);
        prev.overage_credit_grant_cache = Some(cache);
    });
}

/// Format the grant amount for display. Returns null if amount isn't available
/// (not eligible, or currency we don't know how to format).
pub fn format_grant_amount(info: &OverageCreditGrantInfo) -> Option<String> {
    let amount = info.amount_minor_units?;
    let currency = info.currency.as_ref()?;

    // For now only USD; backend may expand later
    if currency.to_uppercase() == "USD" {
        let dollars = amount as f64 / 100.0;
        if dollars.fract() == 0.0 {
            Some(format!("${}", dollars as i64))
        } else {
            Some(format!("${:.2}", dollars))
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_grant_amount_usd_whole() {
        let info = OverageCreditGrantInfo {
            available: true,
            eligible: true,
            granted: false,
            amount_minor_units: Some(250000),
            currency: Some("USD".to_string()),
        };
        let result = format_grant_amount(&info);
        assert_eq!(result, Some("$2500".to_string()));
    }

    #[test]
    fn test_format_grant_amount_usd_cents() {
        let info = OverageCreditGrantInfo {
            available: true,
            eligible: true,
            granted: false,
            amount_minor_units: Some(254999),
            currency: Some("USD".to_string()),
        };
        let result = format_grant_amount(&info);
        assert_eq!(result, Some("$2549.99".to_string()));
    }

    #[test]
    fn test_format_grant_amount_no_amount() {
        let info = OverageCreditGrantInfo {
            available: false,
            eligible: false,
            granted: false,
            amount_minor_units: None,
            currency: None,
        };
        let result = format_grant_amount(&info);
        assert_eq!(result, None);
    }

    #[test]
    fn test_format_grant_amount_unknown_currency() {
        let info = OverageCreditGrantInfo {
            available: true,
            eligible: true,
            granted: false,
            amount_minor_units: Some(1000),
            currency: Some("EUR".to_string()),
        };
        let result = format_grant_amount(&info);
        assert_eq!(result, None);
    }
}