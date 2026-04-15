#![allow(dead_code)]

use std::collections::HashMap;

pub struct RateLimiter {
    requests: HashMap<String, Vec<u64>>,
    max_requests: usize,
    window_ms: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_ms: u64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_ms,
        }
    }

    pub fn allow(&mut self, key: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let timestamps = self
            .requests
            .entry(key.to_string())
            .or_insert_with(Vec::new);

        timestamps.retain(|t| now - t < self.window_ms);

        if timestamps.len() < self.max_requests {
            timestamps.push(now);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2, 1000);
        assert!(limiter.allow("key"));
        assert!(limiter.allow("key"));
        assert!(!limiter.allow("key"));
    }
}
