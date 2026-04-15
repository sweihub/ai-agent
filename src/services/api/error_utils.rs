// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/errorUtils.ts
//! API error utilities
//! Extracts connection error details and formats API errors

use std::collections::HashSet;

/// SSL/TLS error codes from OpenSSL
static SSL_ERROR_CODES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    // Certificate verification errors
    set.insert("UNABLE_TO_VERIFY_LEAF_SIGNATURE");
    set.insert("UNABLE_TO_GET_ISSUER_CERT");
    set.insert("UNABLE_TO_GET_ISSUER_CERT_LOCALLY");
    set.insert("CERT_SIGNATURE_FAILURE");
    set.insert("CERT_NOT_YET_VALID");
    set.insert("CERT_HAS_EXPIRED");
    set.insert("CERT_REVOKED");
    set.insert("CERT_REJECTED");
    set.insert("CERT_UNTRUSTED");
    // Self-signed certificate errors
    set.insert("DEPTH_ZERO_SELF_SIGNED_CERT");
    set.insert("SELF_SIGNED_CERT_IN_CHAIN");
    // Chain errors
    set.insert("CERT_CHAIN_TOO_LONG");
    set.insert("PATH_LENGTH_EXCEEDED");
    // Hostname/altname errors
    set.insert("ERR_TLS_CERT_ALTNAME_INVALID");
    set.insert("HOSTNAME_MISMATCH");
    // TLS handshake errors
    set.insert("ERR_TLS_HANDSHAKE_TIMEOUT");
    set.insert("ERR_SSL_WRONG_VERSION_NUMBER");
    set.insert("ERR_SSL_DECRYPTION_FAILED_OR_BAD_RECORD_MAC");
    set
});

use once_cell::sync::Lazy;

/// Connection error details
#[derive(Debug, Clone)]
pub struct ConnectionErrorDetails {
    pub code: String,
    pub message: String,
    pub is_ssl_error: bool,
}

/// Extracts connection error details from an error message string
pub fn extract_connection_error_details_from_message(msg: &str) -> Option<ConnectionErrorDetails> {
    // Check for common error codes in the message
    let lower = msg.to_lowercase();

    // Check for timeout
    if lower.contains("timed out") || lower.contains("etimedout") {
        return Some(ConnectionErrorDetails {
            code: "ETIMEDOUT".to_string(),
            message: msg.to_string(),
            is_ssl_error: false,
        });
    }

    // Check for SSL/TLS errors
    let is_ssl = lower.contains("ssl") || lower.contains("tls") || lower.contains("certificate");
    if is_ssl {
        // Try to extract specific SSL code
        let code = if lower.contains("self_signed") || lower.contains("self signed") {
            "DEPTH_ZERO_SELF_SIGNED_CERT".to_string()
        } else if lower.contains("certificate has expired") {
            "CERT_HAS_EXPIRED".to_string()
        } else if lower.contains("hostname") || lower.contains("altname") {
            "ERR_TLS_CERT_ALTNAME_INVALID".to_string()
        } else {
            "SSL_ERROR".to_string()
        };

        return Some(ConnectionErrorDetails {
            code,
            message: msg.to_string(),
            is_ssl_error: true,
        });
    }

    // Check for connection reset
    if lower.contains("econnreset") || lower.contains("connection reset") {
        return Some(ConnectionErrorDetails {
            code: "ECONNRESET".to_string(),
            message: msg.to_string(),
            is_ssl_error: false,
        });
    }

    // Check for broken pipe
    if lower.contains("epipe") || lower.contains("broken pipe") {
        return Some(ConnectionErrorDetails {
            code: "EPIPE".to_string(),
            message: msg.to_string(),
            is_ssl_error: false,
        });
    }

    None
}

/// Returns an actionable hint for SSL/TLS errors
pub fn get_ssl_error_hint(error_message: &str) -> Option<String> {
    let details = extract_connection_error_details_from_message(error_message)?;
    if !details.is_ssl_error {
        return None;
    }
    Some(format!(
        "SSL certificate error ({}). If you are behind a corporate proxy or TLS-intercepting firewall, set NODE_EXTRA_CA_CERTS to your CA bundle path, or ask IT to allowlist *.anthropic.com. Run /doctor for details.",
        details.code
    ))
}

/// Strips HTML content (e.g., CloudFlare error pages) from a message string
fn sanitize_message_html(message: &str) -> String {
    let lower = message.to_lowercase();
    if lower.contains("<!DOCTYPE html") || lower.contains("<html") {
        // Case-insensitive check, but use original message with case-insensitive pattern matching
        let title_pattern = regex::Regex::new("(?i)<title>([^<]+)</title>").ok();
        if let Some(re) = title_pattern {
            if let Some(caps) = re.captures(message) {
                if let Some(title) = caps.get(1) {
                    return title.as_str().trim().to_string();
                }
            }
        }
        return String::new();
    }
    message.to_string()
}

/// Detects if an error message contains HTML content
pub fn sanitize_api_error(message: &str) -> String {
    if message.is_empty() {
        return String::new();
    }
    sanitize_message_html(message)
}

/// Shape of deserialized API errors from session JSONL
#[derive(Debug, Clone)]
pub struct NestedApiError {
    pub message: Option<String>,
    pub error: Option<Box<NestedApiError>>,
}

impl<'de> serde::Deserialize<'de> for NestedApiError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let message = value.get("message").and_then(|v| v.as_str()).map(String::from);
        let error = value.get("error").and_then(|v| {
            v.as_object().map(|_| {
                Box::new(NestedApiError {
                    message: v.get("message").and_then(|v| v.as_str()).map(String::from),
                    error: None,
                })
            })
        });
        Ok(NestedApiError { message, error })
    }
}

/// Extract a human-readable message from a deserialized API error
pub fn extract_nested_error_message(error: &serde_json::Value) -> Option<String> {
    // Standard Anthropic API shape: { error: { error: { message } } }
    if let Some(error_obj) = error.get("error") {
        if let Some(inner_error) = error_obj.get("error") {
            if let Some(msg) = inner_error.get("message").and_then(|v| v.as_str()) {
                let sanitized = sanitize_message_html(msg);
                if !sanitized.is_empty() {
                    return Some(sanitized);
                }
            }
        }
        // Bedrock shape: { error: { message } }
        if let Some(msg) = error_obj.get("message").and_then(|v| v.as_str()) {
            let sanitized = sanitize_message_html(msg);
            if !sanitized.is_empty() {
                return Some(sanitized);
            }
        }
    }
    None
}

/// Format an API error for display
pub fn format_api_error(error_message: &str) -> String {
    // Extract connection error details from the message
    let connection_details = extract_connection_error_details_from_message(error_message);

    if let Some(ref details) = connection_details {
        let code = &details.code;

        // Handle timeout errors
        if code == "ETIMEDOUT" {
            return "Request timed out. Check your internet connection and proxy settings".to_string();
        }

        // Handle SSL/TLS errors with specific messages
        if details.is_ssl_error {
            match code.as_str() {
                "UNABLE_TO_VERIFY_LEAF_SIGNATURE"
                | "UNABLE_TO_GET_ISSUER_CERT"
                | "UNABLE_TO_GET_ISSUER_CERT_LOCALLY" => {
                    return "Unable to connect to API: SSL certificate verification failed. Check your proxy or corporate SSL certificates".to_string();
                }
                "CERT_HAS_EXPIRED" => {
                    return "Unable to connect to API: SSL certificate has expired".to_string();
                }
                "CERT_REVOKED" => {
                    return "Unable to connect to API: SSL certificate has been revoked".to_string();
                }
                "DEPTH_ZERO_SELF_SIGNED_CERT" | "SELF_SIGNED_CERT_IN_CHAIN" => {
                    return "Unable to connect to API: Self-signed certificate detected. Check your proxy or corporate SSL certificates".to_string();
                }
                "ERR_TLS_CERT_ALTNAME_INVALID" | "HOSTNAME_MISMATCH" => {
                    return "Unable to connect to API: SSL certificate hostname mismatch".to_string();
                }
                "CERT_NOT_YET_VALID" => {
                    return "Unable to connect to API: SSL certificate is not yet valid".to_string();
                }
                _ => {
                    return format!("Unable to connect to API: SSL error ({})", code);
                }
            }
        }
    }

    if error_message == "Connection error." {
        if let Some(details) = connection_details {
            return format!("Unable to connect to API ({})", details.code);
        }
        return "Unable to connect to API. Check your internet connection".to_string();
    }

    if error_message.is_empty() {
        return "API error (status unknown)".to_string();
    }

    let sanitized_message = sanitize_api_error(error_message);
    if sanitized_message != error_message && !sanitized_message.is_empty() {
        sanitized_message
    } else {
        error_message.to_string()
    }
}

/// Format an API error from a status code and optional message
pub fn format_api_error_from_status(status: Option<u16>, message: Option<&str>) -> String {
    let msg = message.unwrap_or("");
    let sanitized = sanitize_api_error(msg);

    if !sanitized.is_empty() && sanitized != msg {
        return sanitized;
    }

    if let Some(s) = status {
        format!("API error (status {})", s)
    } else {
        "API error".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_message_html_plain() {
        let result = sanitize_message_html("Plain error message");
        assert_eq!(result, "Plain error message");
    }

    #[test]
    fn test_sanitize_message_html_with_title() {
        let html = "<!DOCTYPE html><html><title>Error Page</title></html>";
        let result = sanitize_message_html(html);
        assert_eq!(result, "Error Page");
    }

    #[test]
    fn test_sanitize_message_html_cloudflare() {
        let html = "<!DOCTYPE HTML><HTML><TITLE>Access Denied</TITLE></HTML>";
        let result = sanitize_message_html(html);
        assert_eq!(result, "Access Denied");
    }

    #[test]
    fn test_extract_nested_error_message_standard() {
        let json = serde_json::json!({
            "error": {
                "error": {
                    "message": "test error message"
                }
            }
        });
        let result = extract_nested_error_message(&json);
        assert_eq!(result, Some("test error message".to_string()));
    }

    #[test]
    fn test_extract_nested_error_message_bedrock() {
        let json = serde_json::json!({
            "error": {
                "message": "bedrock error"
            }
        });
        let result = extract_nested_error_message(&json);
        assert_eq!(result, Some("bedrock error".to_string()));
    }

    #[test]
    fn test_format_api_error_timeout() {
        let result = format_api_error("Connection timed out");
        assert!(result.contains("timed out"));
    }

    #[test]
    fn test_format_api_error_from_status() {
        let result = format_api_error_from_status(Some(429), Some("Rate limited"));
        assert!(result.contains("429"));
    }

    #[test]
    fn test_extract_connection_error_details_from_message_timeout() {
        let result = extract_connection_error_details_from_message("Connection timed out");
        assert!(result.is_some());
        let details = result.unwrap();
        assert_eq!(details.code, "ETIMEDOUT");
        assert!(!details.is_ssl_error);
    }

    #[test]
    fn test_extract_connection_error_details_from_message_ssl() {
        let result = extract_connection_error_details_from_message("SSL certificate error");
        assert!(result.is_some());
        let details = result.unwrap();
        assert!(details.is_ssl_error);
    }

    #[test]
    fn test_get_ssl_error_hint() {
        let result = get_ssl_error_hint("SSL certificate error");
        assert!(result.is_some());
    }
}