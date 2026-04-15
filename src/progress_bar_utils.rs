use std::sync::atomic::{AtomicU64, Ordering};

pub struct ProgressBar {
    current: AtomicU64,
    total: u64,
    message: String,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        ProgressBar {
            current: AtomicU64::new(0),
            total,
            message: String::new(),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn increment(&self, delta: u64) {
        self.current.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn set(&self, value: u64) {
        self.current.store(value, Ordering::Relaxed);
    }

    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.current() as f64 / self.total as f64) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.current() >= self.total
    }

    pub fn reset(&self) {
        self.current.store(0, Ordering::Relaxed);
    }
}
