pub struct ExponentialBackoff {
    base_ms: u64,
    max_ms: u64,
    multiplier: f64,
    current: u64,
}

impl ExponentialBackoff {
    pub fn new(base_ms: u64, max_ms: u64) -> Self {
        Self {
            base_ms,
            max_ms,
            multiplier: 2.0,
            current: base_ms,
        }
    }

    pub fn next(&mut self) -> u64 {
        let wait = self.current;
        self.current = (self.current as f64 * self.multiplier) as u64;
        self.current = self.current.min(self.max_ms);
        wait
    }

    pub fn reset(&mut self) {
        self.current = self.base_ms;
    }
}
