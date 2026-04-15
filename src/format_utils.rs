use chrono::{DateTime, Duration, Utc};

pub fn format_timestamp(ts: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now());
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn format_timestamp_iso(ts: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now());
    dt.to_rfc3339()
}

pub fn format_duration(seconds: i64) -> String {
    let dur = Duration::seconds(seconds);
    let days = dur.num_days();
    let hours = dur.num_hours() % 24;
    let mins = dur.num_minutes() % 60;
    let secs = dur.num_seconds() % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();

    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result
}
