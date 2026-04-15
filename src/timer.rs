#![allow(dead_code)]

pub struct Timer {
    start: std::time::Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    pub fn reset(&mut self) {
        self.start = std::time::Instant::now();
    }
}
