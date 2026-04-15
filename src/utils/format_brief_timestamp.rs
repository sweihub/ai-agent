#![allow(dead_code)]

use chrono::{DateTime, Local, TimeZone};
use std::env;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn format_brief_timestamp(iso_string: &str, now: Option<SystemTime>) -> String {
    let now = now.unwrap_or(SystemTime::now());

    let d = match DateTime::parse_from_rfc3339(iso_string) {
        Ok(dt) => dt,
        Err(_) => return String::new(),
    };

    let day_diff = get_day_diff(now, d);
    let days_ago = (day_diff.as_millis() as f64 / 86_400_000.0).round() as i64;

    let dt_chrono = d.with_timezone(&Local);

    if days_ago == 0 {
        dt_chrono.format("%H:%M").to_string()
    } else if days_ago > 0 && days_ago < 7 {
        dt_chrono.format("%A %H:%M").to_string()
    } else {
        dt_chrono.format("%a %b %d %H:%M").to_string()
    }
}

fn get_day_diff(now: SystemTime, d: DateTime<chrono::FixedOffset>) -> Duration {
    let now_secs = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let d_secs = d.timestamp() as u64;

    let now_day = now_secs / 86400;
    let d_day = d_secs / 86400;

    let diff = if now_day > d_day {
        (now_day - d_day) * 86400
    } else {
        (d_day - now_day) * 86400
    };

    Duration::from_secs(diff)
}

fn get_locale() -> Option<String> {
    let raw = env::var("LC_ALL")
        .or_else(|_| env::var("LC_TIME"))
        .or_else(|_| env::var("LANG"))
        .unwrap_or_default();

    if raw.is_empty() || raw == "C" || raw == "POSIX" {
        return None;
    }

    let base = raw.split('.').next().unwrap_or("");
    let base = base.split('@').next().unwrap_or("");

    if base.is_empty() {
        return None;
    }

    let tag = base.replace('_', "-");
    Some(tag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_format_brief_timestamp() {
        let now = SystemTime::now();
        let result = format_brief_timestamp("2024-01-15T10:30:00Z", Some(now));
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_brief_timestamp_invalid() {
        let now = SystemTime::now();
        assert_eq!(format_brief_timestamp("invalid", Some(now)), "");
    }
}
