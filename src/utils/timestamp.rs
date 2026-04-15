// Source: /data/home/swei/claudecode/openclaudecode/src/types/generated/google/protobuf/timestamp.ts
#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn format_timestamp(ts: u64) -> String {
    let secs = ts / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{}d {}h", days, hours % 24)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins % 60)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let ts = current_timestamp_ms();
        assert!(ts > 0);
    }

    #[test]
    fn test_format() {
        assert!(!format_timestamp(0).is_empty());
    }
}
