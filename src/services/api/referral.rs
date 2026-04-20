// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/referral.ts
//! Referral module
//! Handles referral eligibility and guest passes

use std::collections::HashMap;

use crate::utils::http::get_user_agent;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

/// Cache expiration time: 24 hours
const CACHE_EXPIRATION_MS: u64 = 24 * 60 * 60 * 1000;

/// Referral campaign type
#[derive(Debug, Clone)]
pub struct ReferralCampaign(String);

impl Default for ReferralCampaign {
    fn default() -> Self {
        ReferralCampaign("claude_code_guest_pass".to_string())
    }
}

impl ReferralCampaign {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Referral eligibility response
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralEligibilityResponse {
    pub eligible: bool,
    pub remaining_passes: Option<u32>,
    pub referrer_reward: Option<ReferrerRewardInfo>,
}

/// Referral redemptions response
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralRedemptionsResponse {
    pub redemptions: Vec<ReferralRedemption>,
}

/// Referral redemption entry
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferralRedemption {
    pub redeemed_at: String,
    pub guest_email: Option<String>,
}

/// Referrer reward info
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferrerRewardInfo {
    pub currency: String,
    pub amount_minor_units: u64,
}

/// Cache entry for passes eligibility
#[derive(Debug, Clone)]
pub struct PassesEligibilityCacheEntry {
    pub eligible: bool,
    pub remaining_passes: Option<u32>,
    pub referrer_reward: Option<ReferrerRewardInfo>,
    pub timestamp: u64,
}

/// Track in-flight fetch
static FETCH_IN_PROGRESS: Lazy<Mutex<Option<ReferralEligibilityResponse>>> =
    Lazy::new(|| Mutex::new(None));

/// Get current timestamp in milliseconds
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Get OAuth account info from environment
fn get_oauth_account_info() -> Option<OAuthAccountInfo> {
    std::env::var("AI_CODE_ORGANIZATION_UUID")
        .ok()
        .map(|uuid| OAuthAccountInfo {
            organization_uuid: Some(uuid),
        })
}

#[derive(Debug, Clone)]
pub struct OAuthAccountInfo {
    pub organization_uuid: Option<String>,
}

/// Check if user is Claude.ai subscriber
fn is_claude_ai_subscriber() -> bool {
    // Check for OAuth token which indicates a subscriber
    std::env::var("AI_CODE_OAUTH_TOKEN").is_ok()
}

/// Get subscription type from environment or config
fn get_subscription_type() -> Option<String> {
    // Try to get from environment variable
    std::env::var("AI_CODE_SUBSCRIPTION_TYPE").ok()
}

/// Check if should check for passes
fn should_check_for_passes() -> bool {
    let account_info = match get_oauth_account_info() {
        Some(info) => info,
        None => return false,
    };

    account_info.organization_uuid.is_some()
        && is_claude_ai_subscriber()
        && get_subscription_type() == Some("max".to_string())
}

/// Get global config (simplified)
fn get_global_config() -> GlobalConfigRef {
    // TODO: Implement properly
    GlobalConfigRef
}

struct GlobalConfigRef;

impl GlobalConfigRef {
    fn passes_eligibility_cache(&self) -> Option<&HashMap<String, PassesEligibilityCacheEntry>> {
        None
    }
}

/// Save global config
fn save_global_config(_update: impl FnOnce(&mut GlobalConfig)) {
    // TODO: Implement properly
}

#[derive(Debug, Clone, Default)]
pub struct GlobalConfig {
    pub passes_eligibility_cache: HashMap<String, PassesEligibilityCacheEntry>,
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

/// Prepare API request (simplified)
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

/// Get OAuth headers
fn get_oauth_headers(access_token: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));
    headers.insert("User-Agent".to_string(), get_user_agent());
    headers
}

/// Check cached passes eligibility
pub fn check_cached_passes_eligibility() -> CachedPassesResult {
    if !should_check_for_passes() {
        return CachedPassesResult {
            eligible: false,
            needs_refresh: false,
            has_cache: false,
        };
    }

    let org_id = match get_oauth_account_info() {
        Some(info) => info.organization_uuid,
        None => None,
    };

    let org_id = match org_id {
        Some(id) => id,
        None => {
            return CachedPassesResult {
                eligible: false,
                needs_refresh: false,
                has_cache: false,
            };
        }
    };

    // Check cache
    // TODO: Implement proper cache check
    CachedPassesResult {
        eligible: false,
        needs_refresh: true,
        has_cache: false,
    }
}

#[derive(Debug, Clone)]
pub struct CachedPassesResult {
    pub eligible: bool,
    pub needs_refresh: bool,
    pub has_cache: bool,
}

/// Currency symbols
fn get_currency_symbol(currency: &str) -> &str {
    match currency {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "BRL" => "R$",
        "CAD" => "CA$",
        "AUD" => "A$",
        "NZD" => "NZ$",
        "SGD" => "S$",
        _ => "",
    }
}

/// Format credit amount
pub fn format_credit_amount(reward: &ReferrerRewardInfo) -> String {
    let symbol = get_currency_symbol(&reward.currency);
    let amount = reward.amount_minor_units as f64 / 100.0;
    let formatted = if amount % 1.0 == 0.0 {
        amount.to_string()
    } else {
        format!("{:.2}", amount)
    };
    format!("{}{}", symbol, formatted)
}

/// Get cached referrer reward
pub fn get_cached_referrer_reward() -> Option<ReferrerRewardInfo> {
    let org_id = get_oauth_account_info()?.organization_uuid?;
    // TODO: Check cache
    None
}

/// Get cached remaining passes
pub fn get_cached_remaining_passes() -> Option<u32> {
    let org_id = get_oauth_account_info()?.organization_uuid?;
    // TODO: Check cache
    None
}

/// Fetch passes eligibility and store in GlobalConfig
pub async fn fetch_and_store_passes_eligibility() -> Option<ReferralEligibilityResponse> {
    // Return existing promise if fetch is already in progress
    {
        let in_progress = FETCH_IN_PROGRESS.lock().unwrap();
        if let Some(ref response) = *in_progress {
            log::debug!("Passes: Reusing in-flight eligibility fetch");
            return Some(response.clone());
        }
    }

    let org_id = match get_oauth_account_info() {
        Some(info) => info.organization_uuid,
        None => None,
    };

    let org_id = match org_id {
        Some(id) => id,
        None => return None,
    };

    // Fetch eligibility
    let response = fetch_referral_eligibility(ReferralCampaign::default()).await;

    // Store in cache
    if let Ok(ref resp) = response {
        let cache_entry = PassesEligibilityCacheEntry {
            eligible: resp.eligible,
            remaining_passes: resp.remaining_passes,
            referrer_reward: resp.referrer_reward.clone(),
            timestamp: now_ms(),
        };

        // Update global config
        // TODO: Implement properly
        log::debug!(
            "Passes eligibility cached for org {}: {}",
            org_id,
            resp.eligible
        );
    }

    // Store in-flight promise
    if let Ok(ref resp) = response {
        let mut in_progress = FETCH_IN_PROGRESS.lock().unwrap();
        *in_progress = Some(resp.clone());
    }

    response.ok()
}

/// Fetch referral eligibility from API
async fn fetch_referral_eligibility(
    campaign: ReferralCampaign,
) -> Result<ReferralEligibilityResponse, String> {
    let request = prepare_api_request().await;
    let access_token = request.access_token;
    let org_uuid = request.org_uuid;

    let mut headers = get_oauth_headers(&access_token);
    headers.insert("x-organization-uuid".to_string(), org_uuid.clone());

    let config = get_oauth_config();
    let url = format!(
        "{}/api/oauth/organizations/{}/referral/eligibility",
        config.base_api_url, org_uuid
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(5000))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .headers(
            headers
                .into_iter()
                .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                .collect(),
        )
        .query(&[("campaign", campaign.as_str())])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<ReferralEligibilityResponse>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch referral redemptions from API
pub async fn fetch_referral_redemptions(
    campaign: Option<String>,
) -> Result<ReferralRedemptionsResponse, String> {
    let request = prepare_api_request().await;
    let access_token = request.access_token;
    let org_uuid = request.org_uuid;

    let mut headers = get_oauth_headers(&access_token);
    headers.insert("x-organization-uuid".to_string(), org_uuid.clone());

    let config = get_oauth_config();
    let url = format!(
        "{}/api/oauth/organizations/{}/referral/redemptions",
        config.base_api_url, org_uuid
    );

    let campaign_str = campaign.unwrap_or_else(|| "claude_code_guest_pass".to_string());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(10000))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .headers(
            headers
                .into_iter()
                .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                .collect(),
        )
        .query(&[("campaign", campaign_str)])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<ReferralRedemptionsResponse>()
        .await
        .map_err(|e| e.to_string())
}

/// Get cached or fetch passes eligibility
pub async fn get_cached_or_fetch_passes_eligibility() -> Option<ReferralEligibilityResponse> {
    if !should_check_for_passes() {
        return None;
    }

    let org_id = match get_oauth_account_info() {
        Some(info) => info.organization_uuid,
        None => None,
    };

    let org_id = match org_id {
        Some(id) => id,
        None => return None,
    };

    // Check cache - if none, fetch in background
    // TODO: Implement proper cache check

    log::debug!(
        "Passes: No cache, fetching eligibility in background (command unavailable this session)"
    );

    // Trigger background fetch
    let _ = fetch_and_store_passes_eligibility().await;

    None
}

/// Prefetch passes eligibility on startup
pub async fn prefetch_passes_eligibility() {
    // Skip if essential traffic only
    if std::env::var("AI_CODE_PRIVACY_LEVEL")
        .map(|v| v == "essential")
        .unwrap_or(false)
    {
        return;
    }

    let _ = get_cached_or_fetch_passes_eligibility().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referral_campaign_default() {
        let campaign = ReferralCampaign::default();
        assert_eq!(campaign.as_str(), "claude_code_guest_pass");
    }

    #[test]
    fn test_format_credit_amount() {
        let reward = ReferrerRewardInfo {
            currency: "USD".to_string(),
            amount_minor_units: 999,
        };
        let formatted = format_credit_amount(&reward);
        assert_eq!(formatted, "$9.99");
    }

    #[test]
    fn test_format_credit_amount_whole() {
        let reward = ReferrerRewardInfo {
            currency: "USD".to_string(),
            amount_minor_units: 1000,
        };
        let formatted = format_credit_amount(&reward);
        assert_eq!(formatted, "$10");
    }

    #[test]
    fn test_check_cached_passes_not_subscriber() {
        let result = check_cached_passes_eligibility();
        assert!(!result.eligible);
    }
}