//! Retry logic with exponential backoff.
//!
//! Provides retry functionality similar to claude code's withRetry.

use std::fmt::Display;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Default maximum number of retries
pub const DEFAULT_MAX_RETRIES: u32 = 10;

/// Base delay in milliseconds
pub const BASE_DELAY_MS: u64 = 500;

/// Maximum delay cap in milliseconds
pub const MAX_DELAY_MS: u64 = 32000;

/// Error that indicates retries are exhausted
#[derive(Debug)]
pub struct RetryError<E> {
    pub original_error: E,
    pub attempts: u32,
}

impl<E: Display + Clone> Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RetryError: {} after {} attempts",
            self.original_error, self.attempts
        )
    }
}

impl<E: Display + Clone + std::fmt::Debug> std::error::Error for RetryError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Result type for retry operations
pub type RetryResult<T, E> = Result<T, RetryError<E>>;

/// Configuration for retry behavior
pub struct RetryConfig {
    /// Maximum number of retries (default: 10)
    pub max_retries: u32,
    /// Base delay in milliseconds (default: 500)
    pub base_delay_ms: u64,
    /// Maximum delay cap in milliseconds (default: 32000)
    pub max_delay_ms: u64,
    /// Enable jitter (default: true)
    pub jitter: bool,
    /// Retry on specific error conditions (takes error message as string)
    pub should_retry: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl RetryConfig {
    /// Create default retry config
    pub fn new() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            base_delay_ms: BASE_DELAY_MS,
            max_delay_ms: MAX_DELAY_MS,
            jitter: true,
            should_retry: None,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate retry delay with exponential backoff and optional jitter
pub fn get_retry_delay(attempt: u32, retry_after_ms: Option<u64>, config: &RetryConfig) -> u64 {
    // If retry-after header is provided, use it directly
    if let Some(retry_after) = retry_after_ms {
        return retry_after;
    }

    // Exponential backoff: base * 2^(attempt-1)
    let base_delay = config
        .base_delay_ms
        .saturating_mul(2u64.saturating_pow(attempt - 1));
    let delay = base_delay.min(config.max_delay_ms);

    // Add jitter (25% of base delay)
    if config.jitter {
        let jitter = (delay as f64 * 0.25 * rand_jitter()) as u64;
        delay + jitter
    } else {
        delay
    }
}

/// Simple random jitter between 0 and 1
fn rand_jitter() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f64) / (u32::MAX as f64)
}

/// Retry an async operation with exponential backoff
///
/// # Arguments
/// * `operation` - The async operation to retry
/// * `config` - Retry configuration
///
/// # Returns
/// * `Ok(T)` - Success
/// * `Err(RetryError<E>)` - All retries exhausted
pub async fn retry_async<T, E, F, Fut>(mut operation: F, config: RetryConfig) -> RetryResult<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + Clone,
{
    let mut last_error: Option<E> = None;

    for attempt in 1..=config.max_retries + 1 {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e.clone());

                // Check if we should retry this error
                if let Some(should_retry) = &config.should_retry {
                    let error_str = format!("{}", e);
                    if !should_retry(&error_str) {
                        return Err(RetryError {
                            original_error: e,
                            attempts: attempt,
                        });
                    }
                }

                // Don't delay on the last attempt
                if attempt <= config.max_retries {
                    let delay = get_retry_delay(attempt, None, &config);
                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(RetryError {
        original_error: last_error.unwrap_or_else(|| {
            panic!("retry_async called with max_retries=0 and no error occurred")
        }),
        attempts: config.max_retries + 1,
    })
}

/// Retry an async operation with exponential backoff and retry-after support
///
/// # Arguments
/// * `operation` - The async operation to retry (receives attempt number)
/// * `config` - Retry configuration
/// * `get_retry_after` - Extract retry-after from error (returns milliseconds)
pub async fn retry_with_retry_after<T, E, F, Fut>(
    mut operation: F,
    config: RetryConfig,
    get_retry_after: impl Fn(&E) -> Option<u64>,
) -> RetryResult<T, E>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + Clone,
{
    let mut last_error: Option<E> = None;

    for attempt in 1..=config.max_retries + 1 {
        match operation(attempt).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e.clone());

                // Check if we should retry this error
                if let Some(should_retry) = &config.should_retry {
                    let error_str = format!("{}", e);
                    if !should_retry(&error_str) {
                        return Err(RetryError {
                            original_error: e,
                            attempts: attempt,
                        });
                    }
                }

                // Don't delay on the last attempt
                if attempt <= config.max_retries {
                    let retry_after_ms = get_retry_after(&e);
                    let delay = get_retry_delay(attempt, retry_after_ms, &config);
                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(RetryError {
        original_error: last_error.unwrap_or_else(|| {
            panic!("retry_with_retry_after called with max_retries=0 and no error occurred")
        }),
        attempts: config.max_retries + 1,
    })
}

/// Check if an error is a rate limit error (429)
pub fn is_rate_limit_error(error: &str) -> bool {
    error.contains("429") || error.to_lowercase().contains("rate limit")
}

/// Check if an error is a service unavailable error (529)
pub fn is_service_unavailable_error(error: &str) -> bool {
    error.contains("529") || error.contains("overloaded")
}

/// Check if an error is a temporary error that should be retried
pub fn is_retryable_error(error: &str) -> bool {
    is_rate_limit_error(error)
        || is_service_unavailable_error(error)
        || is_connection_error(error)
        || is_server_error(error)
}

/// Check if an error is a connection error
pub fn is_connection_error(error: &str) -> bool {
    let error_str = error.to_lowercase();
    error_str.contains("connection")
        || error_str.contains("econnreset")
        || error_str.contains("econnrefused")
        || error_str.contains("epipe")
        || error_str.contains("timeout")
}

/// Check if an error is a server error (5xx)
pub fn is_server_error(error: &str) -> bool {
    // Check for 5xx status codes in error message
    error.contains("500")
        || error.contains("501")
        || error.contains("502")
        || error.contains("503")
        || error.contains("504")
}

/// Create a retry config for rate limit errors
pub fn rate_limit_config() -> RetryConfig {
    RetryConfig {
        max_retries: 5,
        base_delay_ms: 1000,
        max_delay_ms: 60000,
        jitter: true,
        should_retry: Some(Box::new(|e| is_rate_limit_error(e))),
    }
}

/// Create a retry config for service unavailable errors
pub fn service_unavailable_config() -> RetryConfig {
    RetryConfig {
        max_retries: 3,
        base_delay_ms: 2000,
        max_delay_ms: 30000,
        jitter: true,
        should_retry: Some(Box::new(|e| is_service_unavailable_error(e))),
    }
}

/// Create a retry config for all retryable errors
pub fn default_retry_config() -> RetryConfig {
    RetryConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success_first_try() {
        let call_count = std::sync::atomic::AtomicU32::new(0);
        let operation = || {
            let call_count = &call_count;
            async move {
                call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, &'static str>("success")
            }
        };

        let result = retry_async(operation, RetryConfig::default()).await;
        assert!(result.is_ok());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let call_count = std::sync::atomic::AtomicU32::new(0);
        let operation = || {
            let call_count = &call_count;
            async move {
                let count = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < 2 {
                    Err("temporary error")
                } else {
                    Ok("success")
                }
            }
        };

        let result = retry_async(operation, RetryConfig::default()).await;
        assert!(result.is_ok());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let call_count = std::sync::atomic::AtomicU32::new(0);
        let operation = || {
            let call_count = &call_count;
            async move {
                call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err::<String, _>("persistent error")
            }
        };

        let config = RetryConfig {
            max_retries: 3,
            ..Default::default()
        };
        let result = retry_async(operation, config).await;
        assert!(result.is_err());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn test_retry_with_should_retry() {
        let operation = || async move { Err::<String, _>("rate limit") };

        let config = RetryConfig {
            max_retries: 3,
            should_retry: Some(Box::new(|e| format!("{}", e).contains("rate limit"))),
            ..Default::default()
        };
        let result = retry_async(operation, config).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_get_retry_delay_exponential() {
        let config = RetryConfig {
            base_delay_ms: 100,
            max_delay_ms: 10000,
            jitter: false,
            ..Default::default()
        };

        assert_eq!(get_retry_delay(1, None, &config), 100);
        assert_eq!(get_retry_delay(2, None, &config), 200);
        assert_eq!(get_retry_delay(3, None, &config), 400);
        assert_eq!(get_retry_delay(4, None, &config), 800);
    }

    #[test]
    fn test_get_retry_delay_max_cap() {
        let config = RetryConfig {
            base_delay_ms: 1000,
            max_delay_ms: 500,
            jitter: false,
            ..Default::default()
        };

        // Should be capped at max_delay_ms
        assert_eq!(get_retry_delay(10, None, &config), 500);
    }

    #[test]
    fn test_get_retry_delay_with_retry_after() {
        let config = RetryConfig::default();

        // Should use retry-after if provided
        let delay = get_retry_delay(1, Some(5000), &config);
        assert_eq!(delay, 5000);
    }

    #[test]
    fn test_is_rate_limit_error() {
        assert!(is_rate_limit_error(&"429 Too Many Requests"));
        assert!(is_rate_limit_error(&"rate limit exceeded"));
        assert!(!is_rate_limit_error(&"404 Not Found"));
    }

    #[test]
    fn test_is_service_unavailable_error() {
        assert!(is_service_unavailable_error(&"529 Service Unavailable"));
        assert!(is_service_unavailable_error(&"server overloaded"));
        assert!(!is_service_unavailable_error(&"400 Bad Request"));
    }

    #[test]
    fn test_is_connection_error() {
        assert!(is_connection_error(&"connection refused"));
        assert!(is_connection_error(&"ECONNRESET"));
        assert!(!is_connection_error(&"404 Not Found"));
    }

    #[test]
    fn test_is_server_error() {
        assert!(is_server_error(&"500 Internal Server Error"));
        assert!(is_server_error(&"503 Service Unavailable"));
        assert!(!is_server_error(&"400 Bad Request"));
    }
}
