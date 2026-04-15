// Source: ~/claudecode/openclaudecode/src/utils/cronJitterConfig.ts
//! GrowthBook-backed cron jitter configuration.
//!
//! Separated from cron_scheduler so the scheduler can be bundled in the
//! Agent SDK public build without pulling in analytics/growthbook and
//! its large transitive dependency set (settings/hooks/config cycle).
//!
//! Usage:
//!   REPL (use_scheduled_tasks): pass `get_jitter_config: get_cron_jitter_config`
//!   Daemon/SDK: omit get_jitter_config -> DEFAULT_CRON_JITTER_CONFIG applies.

#![allow(dead_code)]

use crate::utils::cron_tasks::{CronJitterConfig, DEFAULT_CRON_JITTER_CONFIG};

/// How often to re-fetch tengu_kairos_cron_config from GrowthBook. Short because
/// this is an incident lever — when we push a config change to shed :00 load,
/// we want the fleet to converge within a minute, not on the next process
/// restart. The underlying call is a synchronous cache read; the refresh just
/// clears the memoized entry so the next read triggers a background fetch.
const JITTER_CONFIG_REFRESH_MS: u64 = 60 * 1000;

/// Upper bounds here are defense-in-depth against fat-fingered GrowthBook
/// pushes. Like poll_config, we reject the whole object on any violation
/// rather than partially trusting it — a config with one bad field falls back
/// to DEFAULT_CRON_JITTER_CONFIG entirely.
const HALF_HOUR_MS: u64 = 30 * 60 * 1000;
const THIRTY_DAYS_MS: u64 = 30 * 24 * 60 * 60 * 1000;

/// Validated cron jitter configuration.
#[derive(Debug, Clone)]
pub struct ValidatedCronJitterConfig {
    pub recurring_frac: f64,
    pub recurring_cap_ms: u64,
    pub one_shot_max_ms: u64,
    pub one_shot_floor_ms: u64,
    pub one_shot_minute_mod: u64,
    pub recurring_max_age_ms: u64,
}

impl ValidatedCronJitterConfig {
    /// Validate the config fields.
    fn validate(&self) -> bool {
        self.recurring_frac >= 0.0
            && self.recurring_frac <= 1.0
            && self.recurring_cap_ms <= HALF_HOUR_MS
            && self.one_shot_max_ms <= HALF_HOUR_MS
            && self.one_shot_floor_ms <= HALF_HOUR_MS
            && self.one_shot_minute_mod >= 1
            && self.one_shot_minute_mod <= 60
            && self.recurring_max_age_ms <= THIRTY_DAYS_MS
            && self.one_shot_floor_ms <= self.one_shot_max_ms
    }
}

/// Read `tengu_kairos_cron_config` from GrowthBook, validate, fall back to
/// defaults on absent/malformed/out-of-bounds config. Called from check()
/// every tick via the `get_jitter_config` callback — cheap (synchronous cache
/// hit). Refresh window: JITTER_CONFIG_REFRESH_MS.
///
/// Exported so ops runbooks can point at a single function when documenting
/// the lever, and so tests can spy on it without mocking GrowthBook itself.
///
/// Pass this as `get_jitter_config` when calling create_cron_scheduler in REPL
/// contexts. Daemon/SDK callers omit get_jitter_config and get defaults.
pub fn get_cron_jitter_config() -> CronJitterConfig {
    let raw = get_feature_value_cached_with_refresh(
        "tengu_kairos_cron_config",
        DEFAULT_CRON_JITTER_CONFIG,
        JITTER_CONFIG_REFRESH_MS,
    );

    // Validate the config
    if let Some(validated) = validate_config(&raw) {
        return CronJitterConfig {
            recurring_frac: validated.recurring_frac,
            recurring_cap_ms: validated.recurring_cap_ms,
            one_shot_max_ms: validated.one_shot_max_ms,
            one_shot_floor_ms: validated.one_shot_floor_ms,
            one_shot_minute_mod: validated.one_shot_minute_mod,
            recurring_max_age_ms: validated.recurring_max_age_ms,
        };
    }

    // Fall back to defaults on validation failure
    DEFAULT_CRON_JITTER_CONFIG
}

/// Validate a raw config value into a validated config.
fn validate_config(raw: &CronJitterConfig) -> Option<ValidatedCronJitterConfig> {
    let validated = ValidatedCronJitterConfig {
        recurring_frac: raw.recurring_frac,
        recurring_cap_ms: raw.recurring_cap_ms,
        one_shot_max_ms: raw.one_shot_max_ms,
        one_shot_floor_ms: raw.one_shot_floor_ms,
        one_shot_minute_mod: raw.one_shot_minute_mod,
        recurring_max_age_ms: raw.recurring_max_age_ms,
    };

    if validated.validate() {
        Some(validated)
    } else {
        None
    }
}

/// Get a feature value from GrowthBook with cached refresh.
/// In a full implementation, this would query the GrowthBook SDK.
/// Here we return the default value.
fn get_feature_value_cached_with_refresh(
    _feature_id: &str,
    default: CronJitterConfig,
    _refresh_ms: u64,
) -> CronJitterConfig {
    // Without GrowthBook integration, return the default.
    default
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::cron_tasks::DEFAULT_CRON_JITTER_CONFIG;

    #[test]
    fn test_validate_config_valid() {
        let config = ValidatedCronJitterConfig {
            recurring_frac: 0.1,
            recurring_cap_ms: 5 * 60 * 1000,
            one_shot_max_ms: 10 * 60 * 1000,
            one_shot_floor_ms: 1 * 60 * 1000,
            one_shot_minute_mod: 5,
            recurring_max_age_ms: 7 * 24 * 60 * 60 * 1000,
        };
        assert!(config.validate());
    }

    #[test]
    fn test_validate_config_invalid_frac() {
        let config = ValidatedCronJitterConfig {
            recurring_frac: 1.5, // > 1.0
            recurring_cap_ms: 0,
            one_shot_max_ms: 0,
            one_shot_floor_ms: 0,
            one_shot_minute_mod: 1,
            recurring_max_age_ms: 0,
        };
        assert!(!config.validate());
    }

    #[test]
    fn test_validate_config_floor_exceeds_max() {
        let config = ValidatedCronJitterConfig {
            recurring_frac: 0.1,
            recurring_cap_ms: 0,
            one_shot_max_ms: 1000,
            one_shot_floor_ms: 2000, // floor > max
            one_shot_minute_mod: 1,
            recurring_max_age_ms: 0,
        };
        assert!(!config.validate());
    }

    #[test]
    fn test_get_cron_jitter_config_returns_default() {
        let config = get_cron_jitter_config();
        assert_eq!(config.recurring_frac, DEFAULT_CRON_JITTER_CONFIG.recurring_frac);
    }
}
