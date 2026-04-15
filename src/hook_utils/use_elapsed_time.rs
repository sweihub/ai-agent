//! Elapsed time tracking and formatting utility

use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DEFAULT_UPDATE_INTERVAL_MS: u64 = 1000;

pub struct ElapsedTimeTracker {
    start_time: u64,
    paused_ms: u64,
    end_time: Option<u64>,
    is_running: bool,
}

impl ElapsedTimeTracker {
    pub fn new(start_time: u64) -> Self {
        Self {
            start_time,
            paused_ms: 0,
            end_time: None,
            is_running: true,
        }
    }

    pub fn with_paused_ms(mut self, paused_ms: u64) -> Self {
        self.paused_ms = paused_ms;
        self
    }

    pub fn with_end_time(mut self, end_time: u64) -> Self {
        self.end_time = Some(end_time);
        self.is_running = false;
        self
    }

    pub fn pause(&mut self) {
        self.is_running = false;
    }

    pub fn resume(&mut self) {
        self.is_running = true;
    }

    pub fn get_elapsed_ms(&self) -> u64 {
        let now = self
            .end_time
            .or_else(|| {
                if self.is_running {
                    Some(now_timestamp())
                } else {
                    None
                }
            })
            .unwrap_or(now_timestamp());

        now.saturating_sub(self.start_time)
            .saturating_sub(self.paused_ms)
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

pub fn now_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn format_duration(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

pub fn format_elapsed_time(
    start_time: u64,
    is_running: bool,
    paused_ms: u64,
    end_time: Option<u64>,
) -> String {
    let now = end_time.unwrap_or(if is_running { now_timestamp() } else { 0 });
    let elapsed = now.saturating_sub(start_time).saturating_sub(paused_ms);
    format_duration(elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(1000), "1s");
        assert_eq!(format_duration(30000), "30s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60000), "1m 0s");
        assert_eq!(format_duration(90000), "1m 30s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600000), "1h 0m");
        assert_eq!(format_duration(3660000), "1h 1m");
    }

    #[test]
    fn test_elapsed_time_tracker() {
        let start = now_timestamp();
        let tracker = ElapsedTimeTracker::new(start);
        assert!(tracker.is_running());
        let elapsed = tracker.get_elapsed_ms();
        assert!(elapsed <= 10);
    }
}
