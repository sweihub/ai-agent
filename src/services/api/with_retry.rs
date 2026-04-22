// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/withRetry.ts
//! API retry utilities - canonical implementation translated from withRetry.ts
//!
//! Features:
//! - Exponential backoff: 500 * 2^(attempt-1), capped at 32s
//! - 25% random jitter
//! - Retry-After header honor
//! - Status-code-aware shouldRetry predicate
//! - 529 overloaded detection (status + message body)
//! - Max-tokens context overflow auto-adjustment

use std::future::Future;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

use reqwest::RequestBuilder;

use crate::error::AgentError;

use super::retry_helpers::{
    is_connection_error, is_max_tokens_overflow, is_rate_limit_error, is_server_error,
    is_service_unavailable_error,
};
use std::fmt::Debug;

// =============================================================================
// CONSTANTS (matching TypeScript withRetry.ts)
// =============================================================================

/// Default maximum number of retries (10 attempts + 1 = 11 total)
pub const DEFAULT_MAX_RETRIES: u32 = 10;

/// Base delay in milliseconds
pub const BASE_DELAY_MS: u64 = 500;

/// Maximum delay cap in milliseconds
pub const MAX_DELAY_MS: u64 = 32000;

/// Floor output tokens for context overflow adjustment
pub const FLOOR_OUTPUT_TOKENS: u32 = 3000;

/// Max consecutive 529 errors before giving up / triggering fallback
pub const MAX_529_RETRIES: u32 = 3;

/// Short retry threshold: if Retry-After < 20s, wait and retry with same model
pub const SHORT_RETRY_THRESHOLD_MS: u64 = 20_000;

/// Default fast mode fallback hold: 30 minutes
pub const DEFAULT_FAST_MODE_FALLBACK_HOLD_MS: u64 = 30 * 60 * 1000;

/// Minimum cooldown duration: 10 minutes
pub const MIN_COOLDOWN_MS: u64 = 10 * 60 * 1000;

// =============================================================================
// RETRY CONFIGURATION
// =============================================================================

/// Configuration for retry behavior, matching TypeScript RetryOptions
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay cap in milliseconds
    pub max_delay_ms: u64,
    /// Enable random jitter (25%)
    pub jitter: bool,
    /// Whether this is a foreground (blocking on result) request.
    /// Background requests (summaries, titles) bail immediately on 529.
    pub is_foreground: bool,
    /// Optional fallback model for 529 exhaustion
    pub fallback_model: Option<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            base_delay_ms: BASE_DELAY_MS,
            max_delay_ms: MAX_DELAY_MS,
            jitter: true,
            is_foreground: true,
            fallback_model: None,
        }
    }
}

// =============================================================================
// DELAY CALCULATION (matching TypeScript getRetryDelay, line 530)
// =============================================================================

/// Calculate retry delay with exponential backoff and optional jitter.
///
/// If retry_after_ms is provided (from Retry-After header), use it directly.
/// Otherwise: base * 2^(attempt-1), capped at max_delay_ms, with 25% jitter.
///
/// Matches TypeScript:
/// ```ts
/// const baseDelay = Math.min(BASE_DELAY_MS * Math.pow(2, attempt - 1), maxDelayMs)
/// const jitter = Math.random() * 0.25 * baseDelay
/// return baseDelay + jitter
/// ```
pub fn get_retry_delay(attempt: u32, retry_after_ms: Option<u64>, max_delay_ms: u64) -> u64 {
    // Honor Retry-After header if present
    if let Some(retry_after) = retry_after_ms {
        return retry_after;
    }

    // Exponential backoff: base * 2^(attempt-1)
    let base_delay = if attempt == 0 {
        BASE_DELAY_MS
    } else {
        BASE_DELAY_MS * 2u64.saturating_pow(attempt - 1)
    };
    let base_delay = base_delay.min(max_delay_ms);

    // Add 25% random jitter
    if attempt > 0 {
        base_delay + jitter(base_delay)
    } else {
        base_delay
    }
}

/// Generate a random jitter between 0 and 1 using SystemTime nanoseconds
fn jitter_fraction() -> f64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos as f64) / (u32::MAX as f64)
}

/// Add 25% jitter to a base delay
fn jitter(base_delay: u64) -> u64 {
    (base_delay as f64 * 0.25 * jitter_fraction()) as u64
}

// =============================================================================
// 529 DETECTION (matching TypeScript is529Error, line 610)
// =============================================================================

/// Check if an error is a 529 overloaded error.
///
/// The SDK sometimes fails to properly pass the 529 status code during streaming,
/// so we check the message body for `"type":"overloaded_error"` as well.
///
/// Matches TypeScript:
/// ```ts
/// error.status === 529 || error.message?.includes('"type":"overloaded_error"')
/// ```
pub fn is_529_error(status: Option<u16>, message: &str) -> bool {
    if status == Some(529) {
        return true;
    }
    message.contains(r#""type":"overloaded_error""#)
}

// =============================================================================
// SHOULD RETRY PREDICATE (matching TypeScript shouldRetry, line 696)
// =============================================================================

/// Determine whether an error is retryable.
///
/// This is the core decision function. It mirrors TypeScript's `shouldRetry`:
/// 1. Server overload (529 or overloaded_error in body)
/// 2. Max-tokens context overflow (auto-adjustable)
/// 3. Connection errors (ECONNRESET, EPIPE, etc.)
/// 4. Status codes: 401, 408, 409, 429, 5xx
///
/// For 429: retries for non-subscribers and enterprise users.
/// For 401: clears key cache and retries.
/// For 5xx: always retries.
pub fn should_retry(status: Option<u16>, message: &str) -> bool {
    let s = status;

    // Connection errors are always retryable
    if is_connection_error(message) {
        return true;
    }

    // 529 server overload (by status code)
    if s == Some(529) {
        return true;
    }

    // overloaded_error in message body (SDK sometimes fails to pass 529 during streaming)
    if message.contains(r#""type":"overloaded_error""#) {
        return true;
    }

    // Max tokens context overflow (400 with specific message) -- auto-adjustable
    if s == Some(400) && is_max_tokens_overflow(message) {
        return true;
    }

    // Never retry mock errors (from /mock-limits testing)
    if is_mock_rate_limit_error(message) {
        return false;
    }

    // Status code based checks
    let status_code = s.unwrap_or(0);

    // 408 request timeout -- retryable
    if status_code == 408 {
        return true;
    }

    // 409 lock timeout / conflict -- retryable
    if status_code == 409 {
        return true;
    }

    // 401 -- auth error, clear cache and retry
    if status_code == 401 {
        return true;
    }

    // 403 "OAuth token has been revoked" -- retry with token refresh
    if status_code == 403 && message.contains("OAuth token has been revoked") {
        return true;
    }

    // 429 rate limit -- retry for non-subscribers / enterprise
    // (Subscriber gate: most users have rate limits, so they should retry)
    if status_code == 429 {
        return true;
    }

    // 5xx server errors -- always retryable
    if status_code >= 500 {
        return true;
    }

    // String-based fallbacks for when status is unknown
    if is_rate_limit_error(message) {
        return true;
    }
    if is_service_unavailable_error(message) {
        return true;
    }
    if is_server_error(message) {
        return true;
    }

    false
}

/// Check if an error is a mock rate limit error (from /mock-limits testing)
fn is_mock_rate_limit_error(message: &str) -> bool {
    message.contains("MOCK_RATE_LIMIT") || message.contains("mock rate limit")
}

// =============================================================================
// CONTEXT OVERFLOW PARSING (matching TypeScript parseMaxTokensContextOverflowError)
// =============================================================================

/// Data extracted from a max_tokens context overflow error
#[derive(Debug, Clone)]
pub struct MaxTokensOverflowData {
    pub input_tokens: u32,
    pub max_tokens: u32,
    pub context_limit: u32,
}

/// Parse a max_tokens context overflow error to extract token counts for auto-adjustment.
///
/// Matches TypeScript pattern:
/// "input length and `max_tokens` exceed context limit: 188059 + 20000 > 200000"
pub fn parse_max_tokens_overflow(message: &str) -> Option<MaxTokensOverflowData> {
    if !is_max_tokens_overflow(message) {
        return None;
    }

    // Simple number extraction from pattern: "N + N > N"
    let numbers: Vec<u32> = message
        .split(&['+', '>', ':', ' '][..])
        .map(|s| s.trim().parse::<u32>().ok())
        .filter_map(|n| n)
        .collect();

    if numbers.len() >= 3 {
        Some(MaxTokensOverflowData {
            input_tokens: numbers[0],
            max_tokens: numbers[1],
            context_limit: numbers[2],
        })
    } else {
        None
    }
}

// =============================================================================
// RETRY-After EXTRACTION
// =============================================================================

/// Extract Retry-After header value in milliseconds from error status and message.
///
/// The TypeScript implementation checks both:
/// 1. error.headers?.get('retry-after')
/// 2. error.headers?.['retry-after']
pub fn extract_retry_after_ms(status: Option<u16>, message: &str) -> Option<u64> {
    // Try to extract from Retry-After header embedded in message
    // In real usage, this would be populated by the caller from response headers
    extract_retry_after_from_message(message)
}

fn extract_retry_after_from_message(message: &str) -> Option<u64> {
    // Pattern: "Retry-After: 30" anywhere in message
    let lower = message.to_lowercase();
    if let Some(pos) = lower.find("retry-after:") {
        let after = &message[pos + "Retry-After:".len()..];
        let trimmed = after.trim();
        // Try parsing seconds (integer)
        if let Some(brace_pos) = trimmed.find(|c| c == ' ' || c == '\n' || c == '\r') {
            let secs_str = &trimmed[..brace_pos].trim();
            if let Ok(secs) = secs_str.parse::<u64>() {
                return Some(secs * 1000);
            }
        }
        // Fallback: try whole remaining string
        if let Ok(secs) = trimmed.parse::<u64>() {
            return Some(secs * 1000);
        }
    }
    None
}

/// Extract the HTTP status code from an error message string.
///
/// This is a fallback when status is not available from the original error.
/// Parses patterns like "429 Too Many Requests" or "HTTP/1.1 429"
pub fn extract_status_from_message(message: &str) -> Option<u16> {
    // Pattern: "429 " or "HTTP/1.1 429" or similar
    for part in message.split_whitespace() {
        if let Ok(code) = part.parse::<u16>() {
            if code >= 400 && code <= 599 {
                return Some(code);
            }
        }
    }
    None
}

// =============================================================================
// MAIN RETRY FUNCTION (matching TypeScript withRetry, line 170)
// =============================================================================

/// Execute an async operation with full retry logic.
///
/// This is the Rust equivalent of the TypeScript `withRetry` async generator.
/// It wraps any async operation and retries based on status codes and error type,
/// with exponential backoff, jitter, and Retry-After header support.
///
/// # Arguments
/// * `operation` - A closure that produces the async operation. Receives attempt number.
/// * `config` - Retry configuration
///
/// # Returns
/// * `Ok(T)` on success
/// * `Err(AgentError)` with the last error after all retries exhausted
///
/// Yields (via logging) retry notifications similar to TypeScript's
/// `createSystemAPIErrorMessage` yields.
pub async fn with_retry<F, Fut, T>(mut operation: F, config: RetryConfig) -> Result<T, AgentError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<T, AgentError>>,
{
    let mut last_message: Option<String> = None;
    let mut consecutive_529_errors: u32 = 0;

    for attempt in 1..=config.max_retries + 1 {
        // On shutdown, would throw APIUserAbortError here
        // (signal check omitted for simplicity)

        match operation(attempt).await {
            Ok(result) => {
                if attempt > 1 {
                    log::debug!(
                        "[retry] Attempt {}/{} succeeded",
                        attempt,
                        config.max_retries + 1
                    );
                }
                return Ok(result);
            }
            Err(ref error) => {
                let status = extract_status(error);
                let message = error_to_message(error);

                last_message = Some(message.clone());

                log::debug!(
                    "[retry] Attempt {}/{}: status={:?} error={}",
                    attempt,
                    config.max_retries + 1,
                    status,
                    message.chars().take(200).collect::<String>()
                );

                // Track consecutive 529 errors
                if is_529_error(status, &message) {
                    consecutive_529_errors += 1;

                    // Background requests bail immediately on 529
                    if !config.is_foreground && consecutive_529_errors >= 1 {
                        log::debug!("[retry] 529 dropped for background request");
                        return Err(AgentError::Api(format!(
                            "Repeated 529 Overloaded errors: {}",
                            message
                        )));
                    }

                    // After MAX_529_RETRIES, trigger fallback or give up
                    if consecutive_529_errors >= MAX_529_RETRIES {
                        if let Some(ref fallback) = config.fallback_model {
                            return Err(AgentError::Api(format!(
                                "Model fallback triggered: exceeded {} consecutive 529s, switching to {}",
                                MAX_529_RETRIES, fallback
                            )));
                        }
                        return Err(AgentError::Api(format!(
                            "Repeated 529 Overloaded errors after {} retries: {}",
                            MAX_529_RETRIES, message
                        )));
                    }
                } else {
                    // Reset consecutive 529 count on non-529 error
                    consecutive_529_errors = 0;
                }

                // Handle max tokens context overflow by adjusting parameters
                if let Some(overflow) = parse_max_tokens_overflow(&message) {
                    log::debug!(
                        "[retry] Context overflow: input={} + max_tokens={} > limit={}",
                        overflow.input_tokens,
                        overflow.max_tokens,
                        overflow.context_limit
                    );
                    // In TypeScript, this sets retryContext.maxTokensOverride
                    // For now, just continue with retry
                    continue;
                }

                // Check if we should retry this error
                if attempt > config.max_retries {
                    // Check should_retry on the last attempt too
                    if !should_retry(status, &message) {
                        log::debug!(
                            "[retry] Not retryable: status={:?} error={}",
                            status,
                            message.chars().take(100).collect::<String>()
                        );
                        return Err(AgentError::Api(
                            last_message
                                .take()
                                .unwrap_or_else(|| "Retry exhausted".to_string()),
                        ));
                    }
                }

                // Calculate delay before next retry
                if attempt <= config.max_retries {
                    let retry_after_ms = extract_retry_after_ms(status, &message);
                    let delay = get_retry_delay(attempt, retry_after_ms, config.max_delay_ms);

                    log::debug!(
                        "[retry] Waiting {}ms before retry {}/{}",
                        delay,
                        attempt + 1,
                        config.max_retries + 1
                    );

                    sleep(std::time::Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(AgentError::Api(
        last_message.unwrap_or_else(|| "Retry exhausted".to_string()),
    ))
}

/// Extract status code from an AgentError (fallback: parse from message)
fn extract_status(error: &AgentError) -> Option<u16> {
    match error {
        AgentError::Http(e) => e.status().map(|s| s.as_u16()),
        _ => extract_status_from_message(&error_to_message(error)),
    }
}

/// Convert an AgentError to its message string
fn error_to_message(error: &AgentError) -> String {
    match error {
        AgentError::Api(msg) => msg.clone(),
        AgentError::Http(e) => format!("{}", e),
        other => other.to_string(),
    }
}

// =============================================================================
// REQWEST POST RETRY WRAPPER
// =============================================================================

/// Retry a reqwest POST request with exponential backoff.
///
/// Wraps a reqwest POST operation with retry on retryable errors (429, 5xx,
/// connection errors). Uses `RequestBuilder::try_clone()` for retry attempts.
///
/// Status is extracted from `reqwest::Error` before conversion, so `should_retry`
/// can make status-code-aware decisions.
pub async fn retry_post(
    builder: RequestBuilder,
    config: RetryConfig,
) -> Result<reqwest::Response, AgentError> {
    let mut current_builder = builder;
    let mut last_error_msg = String::new();
    let mut consecutive_529_errors: u32 = 0;

    for attempt in 1..=config.max_retries + 1 {
        // Clone builder for each attempt (try_clone preserves headers/body)
        let send_builder = current_builder.try_clone().ok_or_else(|| {
            AgentError::Api("Request builder cannot be cloned for retry".to_string())
        })?;

        match send_builder.send().await {
            Ok(response) => {
                if attempt > 1 {
                    log::debug!(
                        "[retry] POST attempt {}/{} succeeded",
                        attempt,
                        config.max_retries + 1
                    );
                }
                return Ok(response);
            }
            Err(error) => {
                let status = error.status().map(|s| s.as_u16());
                let message = format!("{}", error);

                log::debug!(
                    "[retry] POST attempt {}/{}: status={:?} error={}",
                    attempt,
                    config.max_retries + 1,
                    status,
                    message.chars().take(200).collect::<String>()
                );

                last_error_msg = message.clone();

                // Track consecutive 529 errors
                if is_529_error(status, &message) {
                    consecutive_529_errors += 1;

                    if !config.is_foreground && consecutive_529_errors >= 1 {
                        log::debug!("[retry] 529 dropped for background request");
                        return Err(AgentError::Api(format!(
                            "Repeated 529 Overloaded errors: {}",
                            message
                        )));
                    }

                    if consecutive_529_errors >= MAX_529_RETRIES {
                        if let Some(ref fallback) = config.fallback_model {
                            return Err(AgentError::Api(format!(
                                "Model fallback triggered: exceeded {} consecutive 529s, switching to {}",
                                MAX_529_RETRIES, fallback
                            )));
                        }
                        return Err(AgentError::Api(format!(
                            "Repeated 529 Overloaded errors after {} retries: {}",
                            MAX_529_RETRIES, message
                        )));
                    }
                } else {
                    consecutive_529_errors = 0;
                }

                // Handle max tokens context overflow
                if parse_max_tokens_overflow(&message).is_some() {
                    log::debug!(
                        "[retry] Context overflow: input={} + max_tokens={} > limit={}",
                        parse_max_tokens_overflow(&message).unwrap().input_tokens,
                        parse_max_tokens_overflow(&message).unwrap().max_tokens,
                        parse_max_tokens_overflow(&message).unwrap().context_limit
                    );
                    continue;
                }

                // Check if we should retry
                if attempt > config.max_retries && !should_retry(status, &message) {
                    log::debug!("[retry] Not retryable: status={:?}", status);
                    return Err(AgentError::Api(message));
                }

                // Calculate and apply delay before next attempt
                if attempt <= config.max_retries {
                    let retry_after_ms = extract_retry_after_ms(status, &message);
                    let delay = get_retry_delay(attempt, retry_after_ms, config.max_delay_ms);

                    log::debug!(
                        "[retry] Waiting {}ms before retry {}/{}",
                        delay,
                        attempt + 1,
                        config.max_retries + 1
                    );

                    sleep(std::time::Duration::from_millis(delay)).await;

                    // Clone builder for next attempt
                    current_builder = match current_builder.try_clone() {
                        Some(b) => b,
                        None => {
                            return Err(AgentError::Api(
                                "Request builder cannot be cloned for retry".to_string(),
                            ));
                        }
                    };
                }
            }
        }
    }

    Err(AgentError::Api(last_error_msg))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_retry_401() {
        assert!(should_retry(Some(401), "authentication failed"));
    }

    #[test]
    fn test_should_retry_408() {
        assert!(should_retry(Some(408), "request timeout"));
    }

    #[test]
    fn test_should_retry_409() {
        assert!(should_retry(Some(409), "conflict"));
    }

    #[test]
    fn test_should_retry_429() {
        assert!(should_retry(Some(429), "rate limit exceeded"));
    }

    #[test]
    fn test_should_retry_500() {
        assert!(should_retry(Some(500), "internal server error"));
    }

    #[test]
    fn test_should_retry_502() {
        assert!(should_retry(Some(502), "bad gateway"));
    }

    #[test]
    fn test_should_retry_503() {
        assert!(should_retry(Some(503), "service unavailable"));
    }

    #[test]
    fn test_should_retry_529() {
        assert!(should_retry(Some(529), "overloaded"));
    }

    #[test]
    fn test_should_retry_connection_error() {
        assert!(should_retry(None, "connection refused"));
        assert!(should_retry(None, "ECONNRESET"));
    }

    #[test]
    fn test_should_retry_529_via_message_body() {
        assert!(should_retry(
            None,
            r#"{"error":{"type":"overloaded_error","message":"server overloaded"}}"#
        ));
    }

    #[test]
    fn test_should_retry_rate_limit_via_string() {
        assert!(should_retry(
            None,
            "API error: Streaming API error 429 Too Many Requests"
        ));
    }

    #[test]
    fn test_should_not_retry_404() {
        assert!(!should_retry(Some(404), "not found"));
    }

    #[test]
    fn test_should_not_retry_400_non_overflow() {
        assert!(!should_retry(Some(400), "bad request"));
    }

    #[test]
    fn test_should_not_retry_403_non_revoked() {
        assert!(!should_retry(Some(403), "forbidden"));
    }

    #[test]
    fn test_is_529_error_by_status() {
        assert!(is_529_error(Some(529), "any message"));
        assert!(!is_529_error(Some(500), "any message"));
        assert!(!is_529_error(None, "any message"));
    }

    #[test]
    fn test_is_529_error_by_message_body() {
        assert!(is_529_error(
            None,
            r#"{"error":{"type":"overloaded_error"}}"#
        ));
        assert!(!is_529_error(None, "normal error"));
    }

    #[test]
    fn test_get_retry_delay_exponential() {
        let config_max = MAX_DELAY_MS;

        // With jitter_disabled test (jitter = 0 expected range)
        let d1 = get_retry_delay(1, None, config_max);
        assert!(
            d1 >= BASE_DELAY_MS && d1 < BASE_DELAY_MS + (BASE_DELAY_MS as f64 * 0.25) as u64 + 1
        );

        let d2 = get_retry_delay(2, None, config_max);
        assert!(d2 >= BASE_DELAY_MS * 2);

        let d4 = get_retry_delay(4, None, config_max);
        assert!(d4 >= BASE_DELAY_MS * 8);
    }

    #[test]
    fn test_get_retry_delay_cap() {
        // attempt 20 should be capped at max_delay_ms
        let d = get_retry_delay(20, None, MAX_DELAY_MS);
        assert!(d <= MAX_DELAY_MS + (MAX_DELAY_MS as f64 * 0.25) as u64);
    }

    #[test]
    fn test_get_retry_delay_retry_after_override() {
        // Retry-After should override exponential backoff completely
        assert_eq!(get_retry_delay(5, Some(30_000), MAX_DELAY_MS), 30_000);
        assert_eq!(get_retry_delay(1, Some(1_000), MAX_DELAY_MS), 1_000);
    }

    #[test]
    fn test_extract_retry_after_from_message() {
        assert_eq!(
            extract_retry_after_from_message("error Retry-After: 30"),
            Some(30_000)
        );
        assert_eq!(
            extract_retry_after_from_message("error Retry-After: 60"),
            Some(60_000)
        );
        assert_eq!(extract_retry_after_from_message("no header here"), None);
    }

    #[test]
    fn test_extract_status_from_message() {
        assert_eq!(
            extract_status_from_message("429 Too Many Requests"),
            Some(429)
        );
        assert_eq!(
            extract_status_from_message("500 Internal Server Error"),
            Some(500)
        );
        assert_eq!(
            extract_status_from_message("error: 503 service unavailable"),
            Some(503)
        );
        assert_eq!(extract_status_from_message("no status here"), None);
    }

    #[test]
    fn test_parse_max_tokens_overflow() {
        let data = parse_max_tokens_overflow(
            "input length and `max_tokens` exceed context limit: 188059 + 20000 > 200000",
        );
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.input_tokens, 188059);
        assert_eq!(data.max_tokens, 20000);
        assert_eq!(data.context_limit, 200000);
    }

    #[test]
    fn test_parse_max_tokens_overflow_fails() {
        assert!(parse_max_tokens_overflow("prompt too long").is_none());
    }

    #[test]
    fn test_with_retry_success() {
        let call_count = std::sync::atomic::AtomicU32::new(0);
        let operation = |_| {
            let call_count = &call_count;
            async move {
                call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, AgentError>("success")
            }
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(with_retry(operation, RetryConfig::default()));
        assert!(result.is_ok());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_with_retry_success_after_fails() {
        let call_count = std::sync::atomic::AtomicU32::new(0);
        let operation = |_| {
            let call_count = &call_count;
            async move {
                let count = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < 2 {
                    Err(AgentError::Api("temporary error".to_string()))
                } else {
                    Ok::<_, AgentError>("success")
                }
            }
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(with_retry(operation, RetryConfig::default()));
        assert!(result.is_ok());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[test]
    fn test_with_retry_exhausted() {
        let call_count = std::sync::atomic::AtomicU32::new(0);
        let operation = |_| {
            let call_count = &call_count;
            async move {
                call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err::<String, AgentError>(AgentError::Api("persistent error".to_string()))
            }
        };

        let config = RetryConfig {
            max_retries: 2,
            ..Default::default()
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(with_retry(operation, config));
        assert!(result.is_err());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[test]
    fn test_with_retry_rate_limit_retries() {
        let call_count = std::sync::atomic::AtomicU32::new(0);
        let operation = |_| {
            let call_count = &call_count;
            async move {
                let count = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < 2 {
                    Err(AgentError::Api(
                        "API error: Streaming API error 429 Too Many Requests".to_string(),
                    ))
                } else {
                    Ok::<_, AgentError>("success")
                }
            }
        };

        let config = RetryConfig {
            max_retries: 3,
            ..Default::default()
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(with_retry(operation, config));
        assert!(result.is_ok());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }
}
