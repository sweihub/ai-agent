// Source: ~/claudecode/openclaudecode/src/hooks/useElapsedTime.ts
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Notify;

/// Format a duration into a human-readable string.
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    if total_secs < 60 {
        format!("{total_secs}s")
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        if secs == 0 {
            format!("{mins}m")
        } else {
            format!("{mins}m {secs}s")
        }
    } else {
        let hrs = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        if mins == 0 {
            format!("{hrs}h")
        } else {
            format!("{hrs}h {mins}m")
        }
    }
}

/// Returns formatted elapsed time since `start_time`.
///
/// Translation of the React `useElapsedTime` hook.
/// In Rust this is a struct that computes elapsed time on demand,
/// with an optional async ticker for live updates.
pub struct ElapsedTime {
    start_time: Instant,
    is_running: bool,
    update_interval_ms: u64,
    paused_ms: Duration,
    end_time: Option<Instant>,
}

impl ElapsedTime {
    /// Create a new elapsed time tracker.
    ///
    /// - `start_time`: when the timer started (as an Instant)
    /// - `is_running`: whether to actively update the timer
    /// - `update_interval_ms`: how often to trigger updates (default 1000ms)
    /// - `paused_ms`: total paused duration to subtract
    /// - `end_time`: if set, freezes the duration at this timestamp
    pub fn new(
        start_time: Instant,
        is_running: bool,
        update_interval_ms: Option<u64>,
        paused_ms: Duration,
        end_time: Option<Instant>,
    ) -> Self {
        Self {
            start_time,
            is_running,
            update_interval_ms: update_interval_ms.unwrap_or(1000),
            paused_ms,
            end_time,
        }
    }

    /// Create from unix timestamps (in milliseconds).
    pub fn from_unix_ms(
        start_time_ms: u64,
        is_running: bool,
        update_interval_ms: Option<u64>,
        paused_ms: u64,
        end_time_ms: Option<u64>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let elapsed_since_start = now.saturating_sub(start_time_ms);
        let start =
            Instant::now().checked_sub(Duration::from_millis(elapsed_since_start)).unwrap_or(Instant::now());

        let end = end_time_ms.map(|ms| {
            let duration = ms.saturating_sub(start_time_ms);
            start.checked_add(Duration::from_millis(duration)).unwrap_or(Instant::now())
        });

        Self {
            start_time: start,
            is_running,
            update_interval_ms: update_interval_ms.unwrap_or(1000),
            paused_ms: Duration::from_millis(paused_ms),
            end_time: end,
        }
    }

    /// Get the current formatted elapsed time.
    pub fn get(&self) -> String {
        let end = self.end_time.unwrap_or(Instant::now());
        let elapsed = end
            .duration_since(self.start_time)
            .saturating_sub(self.paused_ms);
        format_duration(elapsed)
    }

    /// Subscribe to elapsed time updates.
    ///
    /// Returns a `Notify` that is signaled at the given interval while running.
    /// Translation of the `useSyncExternalStore` subscribe mechanism.
    pub fn subscribe(&self) -> Option<Arc<Notify>> {
        if !self.is_running {
            return None;
        }
        let notify = Arc::new(Notify::new());
        let interval_ms = self.update_interval_ms;
        let notify_clone = Arc::clone(&notify);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
            loop {
                interval.tick().await;
                notify_clone.notify_waiters();
            }
        });
        Some(notify)
    }

    /// Get the raw elapsed duration.
    pub fn elapsed_duration(&self) -> Duration {
        let end = self.end_time.unwrap_or(Instant::now());
        end.duration_since(self.start_time)
            .saturating_sub(self.paused_ms)
    }
}
