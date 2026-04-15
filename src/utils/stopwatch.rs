pub struct Stopwatch {
    start: Option<u64>,
    elapsed: u64,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            start: None,
            elapsed: 0,
        }
    }

    pub fn start(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        self.start = Some(now);
    }

    pub fn stop(&mut self) -> u64 {
        if let Some(start) = self.start.take() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
            let delta = now.saturating_sub(start);
            self.elapsed += delta;
            delta
        } else {
            0
        }
    }

    pub fn reset(&mut self) {
        self.start = None;
        self.elapsed = 0;
    }

    pub fn elapsed_ns(&self) -> u64 {
        self.elapsed
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}
