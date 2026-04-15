//! Environment-less bridge configuration.
//!
//! Translated from openclaudecode/src/bridge/envLessBridgeConfig.ts

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// =============================================================================
// TYPES
// =============================================================================

/// Configuration for the env-less bridge timing and behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnvLessBridgeConfig {
    /// Init-phase backoff (createSession, POST /bridge, recovery /bridge)
    pub init_retry_max_attempts: u32,
    /// Base delay for init retry in milliseconds
    pub init_retry_base_delay_ms: u64,
    /// Jitter fraction for init retry (0.0-1.0)
    pub init_retry_jitter_fraction: f64,
    /// Max delay for init retry in milliseconds
    pub init_retry_max_delay_ms: u64,
    /// Axios timeout for POST /sessions, POST /bridge, POST /archive
    pub http_timeout_ms: u64,
    /// BoundedUUIDSet ring size (echo + re-delivery dedup)
    pub uuid_dedup_buffer_size: u32,
    /// CCRClient worker heartbeat cadence in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Fraction of interval for per-beat jitter
    pub heartbeat_jitter_fraction: f64,
    /// Fire proactive JWT refresh this long before expires_in
    pub token_refresh_buffer_ms: u64,
    /// Archive POST timeout in teardown()
    pub teardown_archive_timeout_ms: u64,
    /// Deadline for onConnect after transport.connect()
    pub connect_timeout_ms: u64,
    /// Semver floor for the env-less bridge path
    pub min_version: &'static str,
    /// Whether to show app upgrade message
    pub should_show_app_upgrade_message: bool,
}

/// Default configuration for env-less bridge.
pub const DEFAULT_ENV_LESS_BRIDGE_CONFIG: EnvLessBridgeConfig = EnvLessBridgeConfig {
    init_retry_max_attempts: 3,
    init_retry_base_delay_ms: 500,
    init_retry_jitter_fraction: 0.25,
    init_retry_max_delay_ms: 4000,
    http_timeout_ms: 10_000,
    uuid_dedup_buffer_size: 2000,
    heartbeat_interval_ms: 20_000,
    heartbeat_jitter_fraction: 0.1,
    token_refresh_buffer_ms: 300_000,
    teardown_archive_timeout_ms: 1500,
    connect_timeout_ms: 15_000,
    min_version: &"0.0.0",
    should_show_app_upgrade_message: false,
};

// =============================================================================
// CONFIGURATION FETCHING (STUB - GrowthBook)
// =============================================================================

static ENV_LESS_BRIDGE_CONFIG: OnceLock<EnvLessBridgeConfig> = OnceLock::new();

/// Get the current version from Cargo.toml
fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Compare two semver strings (a < b).
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

/// Fetch the env-less bridge timing config.
/// Currently returns default config - in production would fetch from GrowthBook.
pub async fn get_env_less_bridge_config() -> EnvLessBridgeConfig {
    ENV_LESS_BRIDGE_CONFIG
        .get_or_init(|| DEFAULT_ENV_LESS_BRIDGE_CONFIG.clone())
        .clone()
}

/// Returns an error message if the current CLI version is below the minimum
/// required for the env-less (v2) bridge path, or None if the version is fine.
pub async fn check_env_less_bridge_min_version() -> Option<String> {
    let cfg = get_env_less_bridge_config().await;
    if !cfg.min_version.is_empty() && version_lt(&get_current_version(), &cfg.min_version) {
        return Some(format!(
            "Your version of AI Code ({}) is too old for Remote Control.\n\
             Version {} or higher is required. Run `ai update` to update.",
            get_current_version(),
            cfg.min_version
        ));
    }
    None
}

/// Whether to nudge users toward upgrading their claude.ai app when a
/// Remote Control session starts.
pub async fn should_show_app_upgrade_message() -> bool {
    // Import from bridge_enabled module
    if !crate::bridge_enabled::is_env_less_bridge_enabled() {
        return false;
    }
    let cfg = get_env_less_bridge_config().await;
    cfg.should_show_app_upgrade_message
}
