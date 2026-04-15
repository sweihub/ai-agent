// Source: ~/claudecode/openclaudecode/src/services/compact/timeBasedMCConfig.ts
//! Time-based microcompact configuration.
//!
//! Triggered when gap since last assistant message exceeds threshold.
//! Fetched from GrowthBook feature flag 'tengu_slate_heron' in TypeScript.

/// Configuration for time-based microcompact
#[derive(Debug, Clone)]
pub struct TimeBasedMCConfig {
    /// Whether time-based MC is enabled
    pub enabled: bool,
    /// Gap threshold in minutes (default: 60)
    pub gap_threshold_minutes: u64,
    /// Number of recent compactable tool results to keep (default: 5)
    pub keep_recent: usize,
}

impl Default for TimeBasedMCConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gap_threshold_minutes: 60,
            keep_recent: 5,
        }
    }
}

/// Get the time-based MC config.
/// In TypeScript, this fetches from GrowthBook. For Rust, use defaults + env override.
pub fn get_time_based_mc_config() -> TimeBasedMCConfig {
    let mut config = TimeBasedMCConfig::default();

    // Allow env override for gap threshold
    if let Ok(val) = std::env::var("AI_MC_GAP_THRESHOLD_MINUTES") {
        if let Ok(parsed) = val.parse::<u64>() {
            if parsed > 0 {
                config.gap_threshold_minutes = parsed;
            }
        }
    }

    // Allow env override for keep_recent
    if let Ok(val) = std::env::var("AI_MC_KEEP_RECENT") {
        if let Ok(parsed) = val.parse::<usize>() {
            if parsed > 0 {
                config.keep_recent = parsed;
            }
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TimeBasedMCConfig::default();
        assert!(config.enabled);
        assert_eq!(config.gap_threshold_minutes, 60);
        assert_eq!(config.keep_recent, 5);
    }

    #[test]
    fn test_get_time_based_mc_config() {
        let config = get_time_based_mc_config();
        assert!(config.enabled);
        assert!(config.gap_threshold_minutes > 0);
        assert!(config.keep_recent > 0);
    }
}
