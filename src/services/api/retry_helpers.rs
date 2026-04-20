// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/withRetry.ts
//! String-based error predicates for retry decisions.
//!
//! These serve as fallback detection when HTTP status codes are unavailable
//! (e.g., errors already converted to plain strings).

/// Check if an error message indicates a rate limit (429)
pub fn is_rate_limit_error(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("429") || lower.contains("rate limit")
}

/// Check if an error message indicates server overload (529)
pub fn is_service_unavailable_error(error: &str) -> bool {
    error.contains("529") || error.contains("overloaded")
}

/// Check if an error message indicates a connection error
pub fn is_connection_error(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("connection")
        || lower.contains("econnreset")
        || lower.contains("econnrefused")
        || lower.contains("epipe")
        || lower.contains("connection reset by peer")
        || lower.contains("broken pipe")
}

/// Check if an error message indicates a 5xx server error
pub fn is_server_error(error: &str) -> bool {
    error.contains("500")
        || error.contains("501")
        || error.contains("502")
        || error.contains("503")
        || error.contains("504")
}

/// Check if an error message indicates a max tokens context overflow
pub fn is_max_tokens_overflow(error: &str) -> bool {
    error.contains("input length and `max_tokens` exceed context limit")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_rate_limit_error() {
        assert!(is_rate_limit_error("429 Too Many Requests"));
        assert!(is_rate_limit_error("rate limit exceeded"));
        assert!(is_rate_limit_error("Streaming API error 429 Too Many Requests"));
        assert!(!is_rate_limit_error("404 Not Found"));
    }

    #[test]
    fn test_is_service_unavailable_error() {
        assert!(is_service_unavailable_error("529 Service Unavailable"));
        assert!(is_service_unavailable_error("server overloaded"));
        assert!(!is_service_unavailable_error("400 Bad Request"));
    }

    #[test]
    fn test_is_connection_error() {
        assert!(is_connection_error("connection refused"));
        assert!(is_connection_error("ECONNRESET"));
        assert!(is_connection_error("Connection reset by peer"));
        assert!(!is_connection_error("404 Not Found"));
    }

    #[test]
    fn test_is_server_error() {
        assert!(is_server_error("500 Internal Server Error"));
        assert!(is_server_error("503 Service Unavailable"));
        assert!(!is_server_error("400 Bad Request"));
    }

    #[test]
    fn test_is_max_tokens_overflow() {
        assert!(is_max_tokens_overflow(
            "input length and `max_tokens` exceed context limit: 188059 + 20000 > 200000"
        ));
        assert!(!is_max_tokens_overflow("prompt too long"));
    }
}
