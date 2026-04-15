// Source: /data/home/swei/claudecode/openclaudecode/src/constants/common.ts
//! Common date utility constants and functions.

use crate::constants::env::ai_code;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// This ensures you get the LOCAL date in ISO format
pub fn get_local_iso_date() -> String {
    // Check for ant-only date override
    if let Ok(override_date) = std::env::var(ai_code::OVERRIDE_DATE) {
        return override_date;
    }

    let now = chrono::Local::now();
    let year = now.format("%Y");
    let month = now.format("%m");
    let day = now.format("%d");
    format!("{}-{}-{}", year, month, day)
}

/// Session start date - memoized for prompt-cache stability
/// Captures the date once at session start
pub static SESSION_START_DATE: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(get_local_iso_date()));

/// Get the session start date (memoized)
pub fn get_session_start_date() -> String {
    SESSION_START_DATE.lock().unwrap().clone()
}

/// Returns "Month YYYY" (e.g. "February 2026") in the user's local timezone.
/// Changes monthly, not daily — used in tool prompts to minimize cache busting.
pub fn get_local_month_year() -> String {
    // Check for override
    if let Ok(override_date) = std::env::var(ai_code::OVERRIDE_DATE) {
        if !override_date.is_empty() {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&override_date, "%Y-%m-%d") {
                return date.format("%B %Y").to_string();
            }
        }
    }
    chrono::Local::now().format("%B %Y").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_local_iso_date() {
        let date = get_local_iso_date();
        assert_eq!(date.len(), 10); // YYYY-MM-DD
        assert!(date.contains('-'));
    }

    #[test]
    fn test_get_session_start_date() {
        let date = get_session_start_date();
        assert_eq!(date.len(), 10);
    }

    #[test]
    fn test_get_local_month_year() {
        let month_year = get_local_month_year();
        assert!(!month_year.is_empty());
    }
}
