//! Bridge poll interval configuration with GrowthBook support.
//!
//! Translated from openclaudecode/src/bridge/pollConfig.ts

use std::sync::OnceLock;

use super::poll_config_defaults::PollIntervalConfig;

/// Cache for poll interval config with 5-minute refresh window.
static POLL_INTERVAL_CONFIG: OnceLock<PollIntervalConfig> = OnceLock::new();

/// Fetch the bridge poll interval config.
/// Currently returns default config - in production would fetch from GrowthBook.
///
/// This is shared by bridgeMain.ts (standalone) and replBridge.ts (REPL) so ops
/// can tune both poll rates fleet-wide with a single config push.
pub fn get_poll_interval_config() -> PollIntervalConfig {
    POLL_INTERVAL_CONFIG
        .get_or_init(|| PollIntervalConfig::default())
        .clone()
}
