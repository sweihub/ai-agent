//! Rate limiting for API requests.
//!
//! Provides rate limit tracking and enforcement similar to claude code.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Utilization percentage (0-100)
    pub utilization: f64,
    /// Reset timestamp (ISO 8601)
    pub resets_at: Option<String>,
    /// Remaining requests
    pub remaining: Option<u32>,
    /// Total requests allowed
    pub limit: Option<u32>,
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: u32,
    /// Maximum tokens per minute
    pub tokens_per_minute: u32,
    /// Enable burst handling
    pub burst: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 100000,
            burst: true,
        }
    }
}

/// Token bucket rate limiter
#[derive(Debug)]
pub struct TokenBucket {
    capacity: u64,
    tokens: u64,
    refill_rate: f64, // tokens per millisecond
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(capacity: u64, refill_per_second: f64) -> Self {
        let refill_rate = refill_per_second / 1000.0; // per millisecond
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume tokens, returns true if successful
    pub fn try_consume(&mut self, tokens: u64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_millis() as f64;
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens as u64).min(self.capacity);
        self.last_refill = Instant::now();
    }

    /// Get current token balance
    pub fn available(&self) -> u64 {
        self.tokens
    }

    /// Reset the bucket
    pub fn reset(&mut self) {
        self.tokens = self.capacity;
        self.last_refill = Instant::now();
    }
}

/// Sliding window rate limiter
#[derive(Debug)]
pub struct SlidingWindow {
    max_requests: u32,
    window_ms: u64,
    requests: Vec<Instant>,
}

impl SlidingWindow {
    /// Create a new sliding window
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_ms: window_duration.as_millis() as u64,
            requests: Vec::new(),
        }
    }

    /// Try to acquire a slot, returns true if successful
    pub fn try_acquire(&mut self) -> bool {
        let now = Instant::now();

        // Remove expired requests
        let window_start = now
            .checked_sub(Duration::from_millis(self.window_ms))
            .unwrap_or(now);

        self.requests.retain(|&t| t > window_start);

        // Check if we can add a new request
        if self.requests.len() < self.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    /// Get time until next slot is available
    pub fn time_until_available(&self) -> Option<Duration> {
        if self.requests.len() < self.max_requests as usize {
            return None;
        }

        let oldest = self.requests.iter().min()?;
        let window_end = oldest
            .checked_add(Duration::from_millis(self.window_ms))
            .unwrap_or(*oldest);

        let now = Instant::now();
        if window_end > now {
            Some(window_end.duration_since(now))
        } else {
            Some(Duration::ZERO)
        }
    }

    /// Get current request count in window
    pub fn current_count(&self) -> u32 {
        let now = Instant::now();
        let window_start = now
            .checked_sub(Duration::from_millis(self.window_ms))
            .unwrap_or(now);

        self.requests.iter().filter(|&&t| t > window_start).count() as u32
    }

    /// Reset the window
    pub fn reset(&mut self) {
        self.requests.clear();
    }
}

/// Rate limiter that combines token bucket and sliding window
#[derive(Debug)]
pub struct RateLimiter {
    request_limiter: SlidingWindow,
    token_limiter: TokenBucket,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: &RateLimitConfig) -> Self {
        let request_limiter =
            SlidingWindow::new(config.requests_per_minute, Duration::from_secs(60));
        let token_limiter = TokenBucket::new(
            config.tokens_per_minute as u64,
            config.tokens_per_minute as f64 / 60.0,
        );

        Self {
            request_limiter,
            token_limiter,
        }
    }

    /// Try to acquire rate limit slot for a request with given token count
    pub fn try_acquire(&mut self, token_count: u64) -> bool {
        self.request_limiter.try_acquire() && self.token_limiter.try_consume(token_count)
    }

    /// Wait until rate limit is available
    pub async fn acquire(&mut self, token_count: u64) {
        while !self.try_acquire(token_count) {
            // Wait for either request slot or token refill
            let request_wait = self.request_limiter.time_until_available();
            let token_wait = if self.token_limiter.available() < token_count {
                // Estimate wait time based on deficit
                let deficit = token_count - self.token_limiter.available();
                let refill_rate = 1000.0 / 60.0; // tokens per ms
                Some(Duration::from_millis((deficit as f64 / refill_rate) as u64))
            } else {
                None
            };

            // Wait for the shorter duration
            let wait_time = match (request_wait, token_wait) {
                (Some(a), Some(b)) => std::cmp::min(a, b),
                (Some(a), None) => a,
                (None, Some(b)) => b,
                (None, None) => Duration::from_millis(100),
            };

            tokio::time::sleep(wait_time).await;
        }
    }

    /// Get current status
    pub fn status(&self) -> RateLimitStatus {
        RateLimitStatus {
            requests_remaining: self.request_limiter.max_requests
                - self.request_limiter.current_count(),
            tokens_remaining: self.token_limiter.available() as u32,
        }
    }

    /// Reset the limiter
    pub fn reset(&mut self) {
        self.request_limiter.reset();
        self.token_limiter.reset();
    }
}

/// Current rate limit status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub requests_remaining: u32,
    pub tokens_remaining: u32,
}

/// Builder for rate limiter
pub struct RateLimiterBuilder {
    config: RateLimitConfig,
}

impl RateLimiterBuilder {
    pub fn new() -> Self {
        Self {
            config: RateLimitConfig::default(),
        }
    }

    pub fn requests_per_minute(mut self, rpm: u32) -> Self {
        self.config.requests_per_minute = rpm;
        self
    }

    pub fn tokens_per_minute(mut self, tpm: u32) -> Self {
        self.config.tokens_per_minute = tpm;
        self
    }

    pub fn burst(mut self, enable: bool) -> Self {
        self.config.burst = enable;
        self
    }

    pub fn build(self) -> RateLimiter {
        RateLimiter::new(&self.config)
    }
}

impl Default for RateLimiterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10, 2.0); // 2 tokens per second

        // Should be able to consume up to capacity
        assert!(bucket.try_consume(5));
        assert!(bucket.try_consume(5));
        assert!(!bucket.try_consume(1)); // Only 0 remaining

        // Wait for refill
        std::thread::sleep(Duration::from_millis(600));
        assert!(bucket.try_consume(1)); // Should have ~1 token
    }

    #[test]
    fn test_sliding_window() {
        let mut window = SlidingWindow::new(3, Duration::from_millis(100));

        // Should allow up to max requests
        assert!(window.try_acquire());
        assert!(window.try_acquire());
        assert!(window.try_acquire());
        assert!(!window.try_acquire()); // Should be full

        // Wait for window to slide
        std::thread::sleep(Duration::from_millis(150));
        assert!(window.try_acquire());
    }

    #[test]
    fn test_sliding_window_count() {
        let mut window = SlidingWindow::new(5, Duration::from_secs(1));

        assert_eq!(window.current_count(), 0);
        window.try_acquire();
        window.try_acquire();
        assert_eq!(window.current_count(), 2);
    }

    #[test]
    fn test_rate_limiter_builder() {
        let limiter = RateLimiterBuilder::new()
            .requests_per_minute(100)
            .tokens_per_minute(50000)
            .build();

        let status = limiter.status();
        assert_eq!(status.requests_remaining, 100);
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let mut limiter = RateLimiterBuilder::new()
            .requests_per_minute(10)
            .tokens_per_minute(1000)
            .build();

        // Should be able to acquire immediately
        limiter.acquire(100).await;

        let status = limiter.status();
        assert!(status.requests_remaining < 10);
    }
}
