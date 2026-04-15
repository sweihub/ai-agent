use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpsMetrics {
    pub fps: f64,
    pub frame_time_ms: f64,
    pub last_update: i64,
}

impl FpsMetrics {
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            frame_time_ms: 0.0,
            last_update: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn update(&mut self, delta_time_ms: f64) {
        if delta_time_ms > 0.0 {
            self.frame_time_ms = delta_time_ms;
            self.fps = 1000.0 / delta_time_ms;
        }
        self.last_update = chrono::Utc::now().timestamp_millis();
    }

    pub fn is_stale(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        now - self.last_update > 1000
    }
}

impl Default for FpsMetrics {
    fn default() -> Self {
        Self::new()
    }
}
