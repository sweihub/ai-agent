// Source: /data/home/swei/claudecode/openclaudecode/src/utils/format.ts
#![allow(dead_code)]

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub fn truncate_middle(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len < 5 {
        "...".to_string()
    } else {
        let ellipsis = "...";
        let available = max_len - ellipsis.len();
        let half = available / 2;
        let start = &s[..half];
        let end = &s[s.len().saturating_sub(available - half)..];
        format!("{}{}{}", start, ellipsis, end)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_duration_ms(ms: u64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hours = mins / 60;

    if hours > 0 {
        format!("{}h {}m", hours, mins % 60)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs % 60)
    } else if secs > 0 {
        format!("{}.{}s", secs, (ms % 1000) / 100)
    } else {
        format!("{}ms", ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "he...");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
    }
}
