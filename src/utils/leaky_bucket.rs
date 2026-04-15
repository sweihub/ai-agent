pub struct LeakyBucket {
    pub capacity: f64,
    pub rate: f64,
    pub tokens: f64,
    pub last_update: u64,
}

impl LeakyBucket {
    pub fn new(capacity: f64, rate: f64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            capacity,
            rate,
            tokens: capacity,
            last_update: now,
        }
    }

    fn refill(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let elapsed = now.saturating_sub(self.last_update) as f64;
        self.tokens = (self.tokens + elapsed * self.rate / 1000.0).min(self.capacity);
        self.last_update = now;
    }

    pub fn try_acquire(&mut self, cost: f64) -> bool {
        self.refill();

        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else {
            false
        }
    }

    pub fn available(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}
