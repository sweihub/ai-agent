pub struct LinearBackoff {
    current_ms: u64,
    increment_ms: u64,
    max_ms: u64,
}

impl LinearBackoff {
    pub fn new(increment_ms: u64, max_ms: u64) -> Self {
        Self {
            current_ms: 0,
            increment_ms,
            max_ms,
        }
    }

    pub fn next(&mut self) -> u64 {
        let wait = self.current_ms;
        self.current_ms = (self.current_ms + self.increment_ms).min(self.max_ms);
        wait
    }

    pub fn reset(&mut self) {
        self.current_ms = 0;
    }
}
