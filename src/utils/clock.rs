pub struct Clock {
    start_time: u64,
}

impl Clock {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { start_time: now }
    }

    pub fn elapsed_ns(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        now.saturating_sub(self.start_time)
    }

    pub fn elapsed_micros(&self) -> u64 {
        self.elapsed_ns() / 1000
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_ns() / 1_000_000
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed_ns() as f64 / 1_000_000_000.0
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
