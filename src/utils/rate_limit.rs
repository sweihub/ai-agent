pub struct RateLimit {
    pub max_requests: u32,
    pub window_ms: u64,
    pub requests: Vec<u64>,
}

impl RateLimit {
    pub fn new(max_requests: u32, window_ms: u64) -> Self {
        Self {
            max_requests,
            window_ms,
            requests: Vec::new(),
        }
    }

    pub fn try_acquire(&mut self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.requests
            .retain(|t| now.saturating_sub(*t) < self.window_ms);

        if self.requests.len() < self.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    pub fn remaining(&self) -> u32 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let active = self
            .requests
            .iter()
            .filter(|t| now.saturating_sub(**t) < self.window_ms)
            .count();
        self.max_requests.saturating_sub(active as u32)
    }

    pub fn reset(&mut self) {
        self.requests.clear();
    }
}
