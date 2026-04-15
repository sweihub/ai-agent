//! Bridge mode entitlement checks.
//!
//! Translated from openclaudecode/src/bridge/bridgeEnabled.ts

use crate::constants::env::ai;
use std::collections::HashMap;
use std::sync::OnceLock;

// =============================================================================
// BUILD-TIME FEATURE FLAGS
// =============================================================================

static BRIDGE_MODE: OnceLock<bool> = OnceLock::new();
static CCR_AUTO_CONNECT: OnceLock<bool> = OnceLock::new();
static CCR_MIRROR: OnceLock<bool> = OnceLock::new();

fn is_bridge_mode_enabled() -> bool {
    *BRIDGE_MODE.get_or_init(|| {
        std::env::var(ai::BRIDGE_MODE)
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    })
}

fn is_ccr_auto_connect_enabled() -> bool {
    *CCR_AUTO_CONNECT.get_or_init(|| {
        std::env::var(ai::CCR_AUTO_CONNECT)
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    })
}

fn is_ccr_mirror_feature_enabled() -> bool {
    *CCR_MIRROR.get_or_init(|| {
        std::env::var(ai::CCR_MIRROR)
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false)
    })
}

// =============================================================================
// AUTH HELPER FUNCTIONS
// =============================================================================

fn is_claude_ai_subscriber() -> bool {
    crate::session_history::get_bridge_access_token().is_some() && has_profile_scope()
}

fn has_profile_scope() -> bool {
    crate::session_history::get_bridge_access_token().is_some()
}

fn get_oauth_account_info() -> Option<OauthAccountInfo> {
    crate::session_history::get_bridge_access_token().map(|_| OauthAccountInfo {
        organization_uuid: std::env::var(ai::ORGANIZATION_UUID).ok(),
        organization_name: None,
        email_address: None,
    })
}

#[derive(Debug, Clone)]
pub struct OauthAccountInfo {
    pub organization_uuid: Option<String>,
    pub organization_name: Option<String>,
    pub email_address: Option<String>,
}

// =============================================================================
// GROWTHBOOK STUB FUNCTIONS
// =============================================================================

static GROWTHBOOK_CACHE: OnceLock<HashMap<String, serde_json::Value>> = OnceLock::new();

fn get_growthbook_cache() -> &'static HashMap<String, serde_json::Value> {
    GROWTHBOOK_CACHE.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert("tengu_ccr_bridge".to_string(), serde_json::json!(false));
        map.insert("tengu_bridge_repl_v2".to_string(), serde_json::json!(false));
        map.insert(
            "tengu_bridge_repl_v2_cse_shim_enabled".to_string(),
            serde_json::json!(true),
        );
        map.insert("tengu_cobalt_harbor".to_string(), serde_json::json!(false));
        map.insert("tengu_ccr_mirror".to_string(), serde_json::json!(false));
        map.insert(
            "tengu_bridge_min_version".to_string(),
            serde_json::json!({ "minVersion": "0.0.0" }),
        );
        map
    })
}

fn get_feature_value_cached<T: serde::de::DeserializeOwned>(feature: &str, default: T) -> T {
    let cache = get_growthbook_cache();
    cache
        .get(feature)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(default)
}

async fn check_gate_cached_or_blocking(gate: &str) -> bool {
    get_feature_value_cached(gate, false)
}

fn get_dynamic_config_cached<T: serde::de::DeserializeOwned>(config_name: &str, default: T) -> T {
    get_feature_value_cached(config_name, default)
}

// =============================================================================
// VERSION CHECK
// =============================================================================

fn version_lt(a: &str, b: &str) -> bool {
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    for (av, bv) in a_parts.iter().zip(b_parts.iter()) {
        if av < bv {
            return true;
        }
        if av > bv {
            return false;
        }
    }
    false
}

fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// =============================================================================
// ENVIRONMENT UTILITIES
// =============================================================================

fn is_env_truthy(env_var: &str) -> bool {
    if env_var.is_empty() {
        return false;
    }
    let binding = env_var.to_lowercase();
    let normalized = binding.trim();
    matches!(normalized, "1" | "true" | "yes" | "on")
}

fn is_env_truthy_opt(env_var: Option<String>) -> bool {
    env_var.map(|v| is_env_truthy(&v)).unwrap_or(false)
}

// =============================================================================
// MAIN BRIDGE ENABLED FUNCTIONS
// =============================================================================

pub fn is_bridge_enabled() -> bool {
    if is_bridge_mode_enabled() {
        is_claude_ai_subscriber() && get_feature_value_cached::<bool>("tengu_ccr_bridge", false)
    } else {
        false
    }
}

pub async fn is_bridge_enabled_blocking() -> bool {
    if is_bridge_mode_enabled() {
        is_claude_ai_subscriber() && check_gate_cached_or_blocking("tengu_ccr_bridge").await
    } else {
        false
    }
}

pub async fn get_bridge_disabled_reason() -> Option<String> {
    if is_bridge_mode_enabled() {
        if !is_claude_ai_subscriber() {
            return Some("Remote Control requires a claude.ai subscription. Run `ai auth login` to sign in with your claude.ai account.".to_string());
        }
        if !has_profile_scope() {
            return Some("Remote Control requires a full-scope login token. Long-lived tokens (from `ai setup-token` or AI_OAUTH_TOKEN) are limited to inference-only for security reasons. Run `ai auth login` to use Remote Control.".to_string());
        }
        if !get_oauth_account_info()
            .and_then(|info| info.organization_uuid)
            .is_some()
        {
            return Some("Unable to determine your organization for Remote Control eligibility. Run `ai auth login` to refresh your account information.".to_string());
        }
        if !check_gate_cached_or_blocking("tengu_ccr_bridge").await {
            return Some("Remote Control is not yet enabled for your account.".to_string());
        }
        None
    } else {
        Some("Remote Control is not available in this build.".to_string())
    }
}

pub fn is_env_less_bridge_enabled() -> bool {
    if is_bridge_mode_enabled() {
        get_feature_value_cached::<bool>("tengu_bridge_repl_v2", false)
    } else {
        false
    }
}

pub fn is_cse_shim_enabled() -> bool {
    if is_bridge_mode_enabled() {
        get_feature_value_cached::<bool>("tengu_bridge_repl_v2_cse_shim_enabled", true)
    } else {
        true
    }
}

pub fn check_bridge_min_version() -> Option<String> {
    if is_bridge_mode_enabled() {
        #[derive(serde::Deserialize)]
        struct MinVersionConfig {
            min_version: String,
        }
        let config = get_dynamic_config_cached::<MinVersionConfig>(
            "tengu_bridge_min_version",
            MinVersionConfig {
                min_version: "0.0.0".to_string(),
            },
        );
        if !config.min_version.is_empty() && version_lt(&get_current_version(), &config.min_version)
        {
            return Some(format!("Your version of AI Code ({}) is too old for Remote Control.\nVersion {} or higher is required. Run `ai update` to update.", get_current_version(), config.min_version));
        }
    }
    None
}

pub fn get_ccr_auto_connect_default() -> bool {
    if is_ccr_auto_connect_enabled() {
        get_feature_value_cached::<bool>("tengu_cobalt_harbor", false)
    } else {
        false
    }
}

pub fn is_ccr_mirror_enabled() -> bool {
    if is_ccr_mirror_feature_enabled() {
        is_env_truthy_opt(std::env::var(ai::CCR_MIRROR).ok())
            || get_feature_value_cached::<bool>("tengu_ccr_mirror", false)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_version_lt() {
        assert!(version_lt("0.7.0", "0.8.0"));
        assert!(version_lt("0.7.2", "0.7.3"));
        assert!(version_lt("0.6.0", "0.7.0"));
        assert!(!version_lt("0.8.0", "0.7.0"));
        assert!(!version_lt("0.7.0", "0.7.0"));
    }
    #[test]
    fn test_is_env_truthy() {
        assert!(is_env_truthy("1"));
        assert!(is_env_truthy("true"));
        assert!(is_env_truthy("TRUE"));
        assert!(is_env_truthy("yes"));
        assert!(is_env_truthy("on"));
        assert!(!is_env_truthy("0"));
        assert!(!is_env_truthy("false"));
        assert!(!is_env_truthy(""));
        assert!(!is_env_truthy("random"));
    }
}
