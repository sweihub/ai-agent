#![allow(dead_code)]

use std::collections::HashMap;

pub struct SlidingWindow {
    timestamps: Vec<std::time::Instant>,
    window_size: std::time::Duration,
}

impl SlidingWindow {
    pub fn new(window_size: std::time::Duration) -> Self {
        Self {
            timestamps: vec![],
            window_size,
        }
    }

    pub fn try_acquire(&mut self) -> bool {
        let now = std::time::Instant::now();
        self.timestamps
            .retain(|t| now.duration_since(*t) < self.window_size);
        self.timestamps.push(now);
        true
    }
}
