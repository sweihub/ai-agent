use std::time::Duration;

pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    pub fn new(max_attempts: u32) -> Self {
        RetryConfig {
            max_attempts,
            ..Default::default()
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    pub fn with_backoff(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = (self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32)) as u64;
        let delay_ms = delay_ms.min(self.max_delay.as_millis() as u64);
        Duration::from_millis(delay_ms)
    }
}
