#![allow(dead_code)]

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct FpsMetrics {
    pub average_fps: f64,
    pub low_1pct_fps: f64,
}

pub struct FpsTracker {
    frame_durations: Vec<f64>,
    first_render_time: Option<Instant>,
    last_render_time: Option<Instant>,
}

impl FpsTracker {
    pub fn new() -> Self {
        FpsTracker {
            frame_durations: Vec::new(),
            first_render_time: None,
            last_render_time: None,
        }
    }

    pub fn record(&mut self, duration_ms: f64) {
        let now = Instant::now();
        if self.first_render_time.is_none() {
            self.first_render_time = Some(now);
        }
        self.last_render_time = Some(now);
        self.frame_durations.push(duration_ms);
    }

    pub fn get_metrics(&self) -> Option<FpsMetrics> {
        if self.frame_durations.is_empty() {
            return None;
        }

        let first = self.first_render_time?;
        let last = self.last_render_time?;

        let total_time_ms = last.duration_since(first).as_millis() as f64;
        if total_time_ms <= 0.0 {
            return None;
        }

        let total_frames = self.frame_durations.len() as f64;
        let average_fps = total_frames / (total_time_ms / 1000.0);

        let mut sorted = self.frame_durations.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let p99_index = ((sorted.len() as f64 * 0.01).ceil() as usize)
            .saturating_sub(1)
            .max(0);
        let p99_frame_time_ms = sorted.get(p99_index).copied().unwrap_or(0.0);
        let low_1pct_fps = if p99_frame_time_ms > 0.0 {
            1000.0 / p99_frame_time_ms
        } else {
            0.0
        };

        Some(FpsMetrics {
            average_fps: (average_fps * 100.0).round() / 100.0,
            low_1pct_fps: (low_1pct_fps * 100.0).round() / 100.0,
        })
    }
}

impl Default for FpsTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_fps_tracker() {
        let mut tracker = FpsTracker::new();
        tracker.record(16.0);
        tracker.record(17.0);
        thread::sleep(Duration::from_millis(50));
        tracker.record(15.0);

        let metrics = tracker.get_metrics();
        assert!(metrics.is_some());
    }
}
