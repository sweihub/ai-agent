//! Memory age and freshness utilities.
//!
//! Provides functions to calculate memory age and generate freshness warnings.

/// Calculate days elapsed since mtime (floor-rounded).
/// Returns 0 for today, 1 for yesterday, 2+ for older.
/// Negative inputs (future mtime, clock skew) clamp to 0.
pub fn memory_age_days(mtime_ms: i64) -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    ((now - mtime_ms) / 86_400_000).max(0)
}

/// Human-readable age string.
/// Models are poor at date arithmetic — a raw ISO timestamp doesn't trigger
/// staleness reasoning the way "47 days ago" does.
pub fn memory_age(mtime_ms: i64) -> String {
    let d = memory_age_days(mtime_ms);
    if d == 0 {
        "today".to_string()
    } else if d == 1 {
        "yesterday".to_string()
    } else {
        format!("{} days ago", d)
    }
}

/// Plain-text staleness caveat for memories >1 day old.
/// Returns empty string for fresh (today/yesterday) memories — warning there is noise.
pub fn memory_freshness_text(mtime_ms: i64) -> String {
    let d = memory_age_days(mtime_ms);
    if d <= 1 {
        return String::new();
    }

    format!(
        "This memory is {} days old. Memories are point-in-time observations, not live state — claims about code behavior or file:line citations may be outdated. Verify against current code before asserting as fact.",
        d
    )
}

/// Per-memory staleness note wrapped in <system-reminder> tags.
/// Returns empty string for memories ≤ 1 day old.
pub fn memory_freshness_note(mtime_ms: i64) -> String {
    let text = memory_freshness_text(mtime_ms);
    if text.is_empty() {
        return String::new();
    }
    format!("<system-reminder>{}</system-reminder>\n", text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_age_days_today() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert_eq!(memory_age_days(now), 0);
    }

    #[test]
    fn test_memory_age_days_yesterday() {
        let yesterday = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            - 86_400_000;
        assert_eq!(memory_age_days(yesterday), 1);
    }

    #[test]
    fn test_memory_age_days_old() {
        let old = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            - 86_400_000 * 47;
        assert_eq!(memory_age_days(old), 47);
    }

    #[test]
    fn test_memory_age_future() {
        let future = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            + 86_400_000;
        assert_eq!(memory_age_days(future), 0);
    }

    #[test]
    fn test_memory_age() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert_eq!(memory_age(now), "today");
    }

    #[test]
    fn test_memory_freshness_text_fresh() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert!(memory_freshness_text(now).is_empty());
    }

    #[test]
    fn test_memory_freshness_text_old() {
        let old = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            - 86_400_000 * 5;
        let text = memory_freshness_text(old);
        assert!(!text.is_empty());
        assert!(text.contains("5 days old"));
    }

    #[test]
    fn test_memory_freshness_note() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert!(memory_freshness_note(now).is_empty());

        let old = now - 86_400_000 * 5;
        let note = memory_freshness_note(old);
        assert!(note.contains("<system-reminder>"));
    }
}
