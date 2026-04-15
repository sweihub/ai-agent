//! Claude AI limits service - tracks and manages Claude API rate limits.
//!
//! Translates claudeAiLimits.ts from claude code.

use std::collections::HashMap;

/// Quota status types
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum QuotaStatus {
    Allowed,
    AllowedWarning,
    Rejected,
}

impl Default for QuotaStatus {
    fn default() -> Self {
        QuotaStatus::Allowed
    }
}

impl std::fmt::Display for QuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaStatus::Allowed => write!(f, "allowed"),
            QuotaStatus::AllowedWarning => write!(f, "allowed_warning"),
            QuotaStatus::Rejected => write!(f, "rejected"),
        }
    }
}

/// Rate limit types
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RateLimitType {
    FiveHour,
    SevenDay,
    SevenDayOpus,
    SevenDaySonnet,
    Overage,
}

impl std::fmt::Display for RateLimitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitType::FiveHour => write!(f, "five_hour"),
            RateLimitType::SevenDay => write!(f, "seven_day"),
            RateLimitType::SevenDayOpus => write!(f, "seven_day_opus"),
            RateLimitType::SevenDaySonnet => write!(f, "seven_day_sonnet"),
            RateLimitType::Overage => write!(f, "overage"),
        }
    }
}

/// Early warning threshold configuration
#[derive(Debug, Clone)]
pub struct EarlyWarningThreshold {
    /// 0-1 scale: trigger warning when usage >= this
    pub utilization: f64,
    /// 0-1 scale: trigger warning when time elapsed <= this
    pub time_pct: f64,
}

/// Early warning configuration
#[derive(Debug, Clone)]
pub struct EarlyWarningConfig {
    pub rate_limit_type: RateLimitType,
    pub claim_abbrev: &'static str,
    pub window_seconds: u64,
    pub thresholds: Vec<EarlyWarningThreshold>,
}

/// Get early warning configurations in priority order (checked first to last)
/// Used as fallback when server doesn't send surpassed-threshold header
/// Warns users when they're consuming quota faster than the time window allows
pub fn early_warning_configs() -> Vec<EarlyWarningConfig> {
    vec![
        EarlyWarningConfig {
            rate_limit_type: RateLimitType::FiveHour,
            claim_abbrev: "5h",
            window_seconds: 5 * 60 * 60,
            thresholds: vec![EarlyWarningThreshold {
                utilization: 0.9,
                time_pct: 0.72,
            }],
        },
        EarlyWarningConfig {
            rate_limit_type: RateLimitType::SevenDay,
            claim_abbrev: "7d",
            window_seconds: 7 * 24 * 60 * 60,
            thresholds: vec![
                EarlyWarningThreshold {
                    utilization: 0.75,
                    time_pct: 0.6,
                },
                EarlyWarningThreshold {
                    utilization: 0.5,
                    time_pct: 0.35,
                },
                EarlyWarningThreshold {
                    utilization: 0.25,
                    time_pct: 0.15,
                },
            ],
        },
    ]
}

/// Maps claim abbreviations to rate limit types for header-based detection
pub fn early_warning_claim_map() -> HashMap<&'static str, RateLimitType> {
    let mut map = HashMap::new();
    map.insert("5h", RateLimitType::FiveHour);
    map.insert("7d", RateLimitType::SevenDay);
    map.insert("overage", RateLimitType::Overage);
    map
}

/// Rate limit display names
pub fn rate_limit_display_names() -> HashMap<RateLimitType, &'static str> {
    let mut map = HashMap::new();
    map.insert(RateLimitType::FiveHour, "session limit");
    map.insert(RateLimitType::SevenDay, "weekly limit");
    map.insert(RateLimitType::SevenDayOpus, "Opus limit");
    map.insert(RateLimitType::SevenDaySonnet, "Sonnet limit");
    map.insert(RateLimitType::Overage, "extra usage limit");
    map
}

/// Get display name for a rate limit type
pub fn get_rate_limit_display_name(rate_type: &RateLimitType) -> &'static str {
    rate_limit_display_names()
        .get(rate_type)
        .copied()
        .unwrap_or_else(|| match rate_type {
            RateLimitType::FiveHour => "session limit",
            RateLimitType::SevenDay => "weekly limit",
            RateLimitType::SevenDayOpus => "Opus limit",
            RateLimitType::SevenDaySonnet => "Sonnet limit",
            RateLimitType::Overage => "extra usage limit",
        })
}

/// Reason why overage is disabled/rejected
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OverageDisabledReason {
    OverageNotProvisioned,
    OrgLevelDisabled,
    OrgLevelDisabledUntil,
    OutOfCredits,
    SeatTierLevelDisabled,
    MemberLevelDisabled,
    SeatTierZeroCreditLimit,
    GroupZeroCreditLimit,
    MemberZeroCreditLimit,
    OrgServiceLevelDisabled,
    OrgServiceZeroCreditLimit,
    NoLimitsConfigured,
    Unknown,
}

/// Claude AI Limits state
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ClaudeAILimits {
    pub status: QuotaStatus,
    #[serde(rename = "unifiedRateLimitFallbackAvailable")]
    pub unified_rate_limit_fallback_available: bool,
    pub resets_at: Option<u64>,
    #[serde(rename = "rateLimitType")]
    pub rate_limit_type: Option<RateLimitType>,
    pub utilization: Option<f64>,
    #[serde(rename = "overageStatus")]
    pub overage_status: Option<QuotaStatus>,
    #[serde(rename = "overageResetsAt")]
    pub overage_resets_at: Option<u64>,
    #[serde(rename = "overageDisabledReason")]
    pub overage_disabled_reason: Option<OverageDisabledReason>,
    #[serde(rename = "isUsingOverage")]
    pub is_using_overage: Option<bool>,
    #[serde(rename = "surpassedThreshold")]
    pub surpassed_threshold: Option<f64>,
}

impl ClaudeAILimits {
    pub fn default_allowed() -> Self {
        Self {
            status: QuotaStatus::Allowed,
            unified_rate_limit_fallback_available: false,
            is_using_overage: Some(false),
            ..Default::default()
        }
    }
}

/// Raw per-window utilization from response headers
#[derive(Debug, Clone, Default)]
pub struct RawWindowUtilization {
    /// 0-1 fraction
    pub utilization: f64,
    /// unix epoch seconds
    pub resets_at: u64,
}

/// Raw utilization data
#[derive(Debug, Clone, Default)]
pub struct RawUtilization {
    /// Five-hour window utilization (if present)
    pub five_hour: Option<RawWindowUtilization>,
    /// Seven-day window utilization (if present)
    pub seven_day: Option<RawWindowUtilization>,
    /// Seven-day Opus-specific window (if present)
    pub seven_day_opus: Option<RawWindowUtilization>,
    /// Seven-day Sonnet-specific window (if present)
    pub seven_day_sonnet: Option<RawWindowUtilization>,
}

/// Get rate limit error message based on limits and model
pub fn get_rate_limit_error_message(limits: &ClaudeAILimits, _model: &str) -> Option<String> {
    // Check if we should show an error based on the limits
    match limits.status {
        QuotaStatus::Allowed => None,
        QuotaStatus::AllowedWarning => {
            Some("You are approaching your rate limit. Consider using a slower model.".to_string())
        }
        QuotaStatus::Rejected => {
            // Build specific message based on rate limit type
            if let Some(rate_type) = &limits.rate_limit_type {
                let display_name = get_rate_limit_display_name(rate_type);

                let reset_msg = if let Some(resets_at) = limits.resets_at {
                    let reset_time = chrono::DateTime::from_timestamp(resets_at as i64, 0)
                        .map(|dt| dt.format("%H:%M").to_string())
                        .unwrap_or_else(|| "soon".to_string());
                    format!(" Please try again around {}.", reset_time)
                } else {
                    String::new()
                };

                Some(format!(
                    "Rate limit exceeded for your {}.{}",
                    display_name, reset_msg
                ))
            } else {
                Some("Rate limit exceeded. Please try again later.".to_string())
            }
        }
    }
}

/// Calculate token warning state
pub fn calculate_token_warning_state(limits: &ClaudeAILimits) -> Option<EarlyWarningThreshold> {
    // Get the current time as a fraction of the window
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as f64;

    for config in early_warning_configs() {
        if let Some(resets_at) = limits.resets_at {
            let window = config.window_seconds as f64;
            let elapsed = resets_at as f64 - now;
            let time_pct = 1.0 - (elapsed / window);

            let utilization = limits.utilization.unwrap_or(0.0);

            for threshold in &config.thresholds {
                if utilization >= threshold.utilization && time_pct <= threshold.time_pct {
                    return Some(threshold.clone());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_early_warning_configs() {
        let configs = early_warning_configs();
        assert!(!configs.is_empty());

        let five_hour = configs
            .iter()
            .find(|c| matches!(c.rate_limit_type, RateLimitType::FiveHour));
        assert!(five_hour.is_some());
    }

    #[test]
    fn test_claim_map() {
        let map = early_warning_claim_map();
        assert_eq!(map.get("5h"), Some(&RateLimitType::FiveHour));
        assert_eq!(map.get("7d"), Some(&RateLimitType::SevenDay));
    }

    #[test]
    fn test_claude_ai_limits_default() {
        let limits = ClaudeAILimits::default();
        assert_eq!(limits.status, QuotaStatus::Allowed);
    }

    #[test]
    fn test_rate_limit_display_name() {
        assert_eq!(
            get_rate_limit_display_name(&RateLimitType::FiveHour),
            "session limit"
        );
    }
}
