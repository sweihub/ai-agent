#![allow(dead_code)]

use std::collections::HashMap;

pub struct RateLimiter {
    requests: HashMap<String, Vec<std::time::Instant>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window_secs,
        }
    }

    pub fn allow(&mut self, key: &str) -> bool {
        let now = std::time::Instant::now();
        let entry = self
            .requests
            .entry(key.to_string())
            .or_insert_with(Vec::new);
        entry.retain(|t| now.duration_since(*t).as_secs() < self.window_secs);
        if entry.len() < self.max_requests {
            entry.push(now);
            true
        } else {
            false
        }
    }
}
