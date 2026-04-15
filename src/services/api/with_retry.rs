//! API retry utilities - translated from withRetry.ts

use std::future::Future;
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay_ms: u64,
    /// Maximum delay between retries
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// HTTP status codes that should trigger a retry
    pub retryable_status_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            retryable_status_codes: vec![429, 500, 502, 503, 504],
        }
    }
}

/// Result type for retry operations
pub enum RetryResult<T> {
    Success(T),
    RetriesExhausted(T),
    Error(String),
}

/// Check if a status code is retryable
pub fn is_retryable_status(code: u16, config: &RetryConfig) -> bool {
    config.retryable_status_codes.contains(&code)
}

/// Calculate the delay for the next retry with exponential backoff
pub fn calculate_delay(attempt: u32, config: &RetryConfig) -> Duration {
    let delay = config.initial_delay_ms as f64 * config.backoff_multiplier.powi(attempt as i32);
    let delay = delay.min(config.max_delay_ms as f64);
    Duration::from_millis(delay as u64)
}

/// Execute an async operation with retry logic
pub async fn with_retry<T, E, F, Fut>(operation: F, config: RetryConfig) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut last_error: Option<E> = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    // Log successful retry
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = Some(e);

                if attempt < config.max_retries {
                    let delay = calculate_delay(attempt, &config);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

/// Execute an async operation with retry logic and retry-after support
pub async fn with_retry_after<T, E, F, Fut>(operation: F, config: RetryConfig) -> Result<T, E>
where
    F: Fn(Option<u64>) -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut last_error: Option<E> = None;
    let mut retry_after: Option<u64> = None;

    for attempt in 0..=config.max_retries {
        let delay = retry_after.or_else(|| {
            if attempt > 0 {
                Some(calculate_delay(attempt - 1, &config).as_millis() as u64)
            } else {
                None
            }
        });

        match operation(delay).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt < config.max_retries {
                    // In a real implementation, we'd parse the retry-after header
                    // from the error response
                    let delay = calculate_delay(attempt, &config);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
    }

    #[test]
    fn test_is_retryable_status() {
        let config = RetryConfig::default();
        assert!(is_retryable_status(429, &config));
        assert!(is_retryable_status(503, &config));
        assert!(!is_retryable_status(400, &config));
        assert!(!is_retryable_status(200, &config));
    }

    #[test]
    fn test_calculate_delay() {
        let config = RetryConfig::default();

        let delay0 = calculate_delay(0, &config);
        assert_eq!(delay0, Duration::from_millis(1000));

        let delay1 = calculate_delay(1, &config);
        assert_eq!(delay1, Duration::from_millis(2000));

        let delay2 = calculate_delay(2, &config);
        assert_eq!(delay2, Duration::from_millis(4000));
    }
}
