//! Rate limit messages - centralized rate limit message generation.
//!
////! Translates rateLimitMessages.ts from claude code.

use crate::services::claude_ai_limits::{ClaudeAILimits, QuotaStatus, RateLimitType};

pub const RATE_LIMIT_ERROR_PREFIXES: &[&str] = &[
    "You've hit your",
    "You've used",
    "You're now using extra usage",
    "You're close to",
    "You're out of extra usage",
];

pub fn is_rate_limit_error_message(text: &str) -> bool {
    RATE_LIMIT_ERROR_PREFIXES
        .iter()
        .any(|prefix| text.starts_with(prefix))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct RateLimitMessage {
    pub message: String,
    pub severity: MessageSeverity,
}

pub fn get_rate_limit_message(limits: &ClaudeAILimits, _model: &str) -> Option<RateLimitMessage> {
    if limits.is_using_overage == Some(true) {
        if limits.overage_status == Some(QuotaStatus::AllowedWarning) {
            return Some(RateLimitMessage {
                message: "You're close to your extra usage spending limit".to_string(),
                severity: MessageSeverity::Warning,
            });
        }
        return None;
    }

    if limits.status == QuotaStatus::Rejected {
        return Some(RateLimitMessage {
            message: get_limit_reached_text(limits, _model),
            severity: MessageSeverity::Error,
        });
    }

    if limits.status == QuotaStatus::AllowedWarning {
        const WARNING_THRESHOLD: f64 = 0.7;

        if let Some(util) = limits.utilization {
            if util < WARNING_THRESHOLD {
                return None;
            }
        }

        let text = get_early_warning_text(limits);
        if let Some(text) = text {
            return Some(RateLimitMessage {
                message: text,
                severity: MessageSeverity::Warning,
            });
        }
    }

    None
}

pub fn get_rate_limit_error_message(limits: &ClaudeAILimits, model: &str) -> Option<String> {
    get_rate_limit_message(limits, model)
        .filter(|m| m.severity == MessageSeverity::Error)
        .map(|m| m.message)
}

pub fn get_rate_limit_warning(limits: &ClaudeAILimits, model: &str) -> Option<String> {
    get_rate_limit_message(limits, model)
        .filter(|m| m.severity == MessageSeverity::Warning)
        .map(|m| m.message)
}

fn get_limit_reached_text(limits: &ClaudeAILimits, model: &str) -> String {
    let reset_message = limits
        .resets_at
        .map(|r| format!(" · resets {}", format_reset_time(r, true)))
        .unwrap_or_default();

    if limits.overage_status == Some(QuotaStatus::Rejected) {
        if limits.overage_disabled_reason
            == Some(crate::services::claude_ai_limits::OverageDisabledReason::OutOfCredits)
        {
            return format!("You're out of extra usage{}", reset_message);
        }
        return format_limit_reached_text("limit", &reset_message, model);
    }

    match &limits.rate_limit_type {
        Some(RateLimitType::SevenDaySonnet) => {
            format_limit_reached_text("Sonnet limit", &reset_message, model)
        }
        Some(RateLimitType::SevenDayOpus) => {
            format_limit_reached_text("Opus limit", &reset_message, model)
        }
        Some(RateLimitType::SevenDay) => {
            format_limit_reached_text("weekly limit", &reset_message, model)
        }
        Some(RateLimitType::FiveHour) => {
            format_limit_reached_text("session limit", &reset_message, model)
        }
        _ => format_limit_reached_text("usage limit", &reset_message, model),
    }
}

fn get_early_warning_text(limits: &ClaudeAILimits) -> Option<String> {
    let limit_name = match &limits.rate_limit_type {
        Some(RateLimitType::SevenDay) => "weekly limit",
        Some(RateLimitType::FiveHour) => "session limit",
        Some(RateLimitType::SevenDayOpus) => "Opus limit",
        Some(RateLimitType::SevenDaySonnet) => "Sonnet limit",
        Some(RateLimitType::Overage) => "extra usage",
        None => return None,
    };

    let used = limits.utilization.map(|u| (u * 100.0) as u32);

    let reset_time = limits.resets_at.map(|r| format_reset_time(r, true));

    let base = match (used, reset_time) {
        (Some(u), Some(t)) => format!("You've used {}% of your {} · resets {}", u, limit_name, t),
        (Some(u), None) => format!("You've used {}% of your {}", u, limit_name),
        (None, Some(t)) => format!("Approaching {} · resets {}", limit_name, t),
        (None, None) => format!("Approaching {}", limit_name),
    };

    Some(base)
}

fn format_limit_reached_text(limit: &str, reset_message: &str, _model: &str) -> String {
    format!("You've hit your {}{}", limit, reset_message)
}

fn format_reset_time(resets_at: u64, _short: bool) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let diff = resets_at.saturating_sub(now);

    if diff < 60 {
        return "less than a minute".to_string();
    }

    let minutes = diff / 60;
    if minutes < 60 {
        return format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" });
    }

    let hours = minutes / 60;
    if hours < 24 {
        return format!("{} hour{}", hours, if hours == 1 { "" } else { "s" });
    }

    let days = hours / 24;
    format!("{} day{}", days, if days == 1 { "" } else { "s" })
}

pub fn get_using_overage_text(limits: &ClaudeAILimits) -> String {
    let reset_time = limits
        .resets_at
        .map(|r| format_reset_time(r, true))
        .unwrap_or_default();

    let limit_name = match &limits.rate_limit_type {
        Some(RateLimitType::FiveHour) => "session limit",
        Some(RateLimitType::SevenDay) => "weekly limit",
        Some(RateLimitType::SevenDayOpus) => "Opus limit",
        Some(RateLimitType::SevenDaySonnet) => "Sonnet limit",
        _ => "",
    };

    if limit_name.is_empty() {
        return "Now using extra usage".to_string();
    }

    let reset_msg = if !reset_time.is_empty() {
        format!(" · Your {} resets {}", limit_name, reset_time)
    } else {
        String::new()
    };

    format!("You're now using extra usage{}", reset_msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rate_limit_error_message() {
        assert!(is_rate_limit_error_message("You've hit your session limit"));
        assert!(is_rate_limit_error_message(
            "You've used 75% of your weekly limit"
        ));
        assert!(is_rate_limit_error_message("You're now using extra usage"));
        assert!(!is_rate_limit_error_message("Something else"));
    }

    #[test]
    fn test_format_reset_time() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        assert_eq!(format_reset_time(now + 30, false), "less than a minute");
        assert_eq!(format_reset_time(now + 120, false), "2 minutes");
        assert_eq!(format_reset_time(now + 3600, false), "1 hour");
        assert_eq!(format_reset_time(now + 86400, false), "1 day");
    }
}
