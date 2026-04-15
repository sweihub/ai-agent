use std::time::{Duration, Instant};

pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    pub fn new(name: &str) -> Self {
        Timer {
            start: Instant::now(),
            name: name.to_string(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_millis(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
}

pub struct RateLimiter {
    max_requests: usize,
    window: Duration,
    requests: Vec<Instant>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        RateLimiter {
            max_requests,
            window,
            requests: Vec::new(),
        }
    }

    pub fn allow_request(&mut self) -> bool {
        let now = Instant::now();
        self.requests.retain(|t| *t + self.window > now);

        if self.requests.len() < self.max_requests {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    pub fn remaining(&self) -> usize {
        let now = Instant::now();
        self.requests.retain(|t| *t + self.window > now);
        self.max_requests.saturating_sub(self.requests.len())
    }

    pub fn reset(&mut self) {
        self.requests.clear();
    }
}
