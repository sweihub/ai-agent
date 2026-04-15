//! Debug utilities for bridge operations.
//!
//! Translated from openclaudecode/src/bridge/debugUtils.ts

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

// =============================================================================
// CONSTANTS
// =============================================================================

const DEBUG_MSG_LIMIT: usize = 2000;

static SECRET_FIELD_NAMES: [&str; 5] = [
    "session_ingress_token",
    "environment_secret",
    "access_token",
    "secret",
    "token",
];

static SECRET_PATTERN: Lazy<Regex> = Lazy::new(|| {
    let pattern = format!(r#""({})"\s*:\s*"([^"]*)""#, SECRET_FIELD_NAMES.join("|"));
    Regex::new(&pattern).unwrap()
});

const REDACT_MIN_LENGTH: usize = 16;

// =============================================================================
// SECRET REDACTION
// =============================================================================

/// Redact secrets from a string, replacing values with [REDACTED] or a partial reveal.
pub fn redact_secrets(s: &str) -> String {
    SECRET_PATTERN
        .replace_all(s, |caps: &regex::Captures| {
            let field = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let value = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            if value.len() < REDACT_MIN_LENGTH {
                return format!(r#""{}":"[REDACTED]""#, field);
            }
            let redacted = format!("{}...{}", &value[..8], &value[value.len() - 4..]);
            format!(r#""{}":"{}""#, field, redacted)
        })
        .to_string()
}

/// Truncate a string for debug logging, collapsing newlines.
pub fn debug_truncate(s: &str) -> String {
    let flat = s.replace('\n', "\\n");
    if flat.len() <= DEBUG_MSG_LIMIT {
        return flat;
    }
    format!("{}... ({} chars)", &flat[..DEBUG_MSG_LIMIT], flat.len())
}

/// Truncate a JSON-serializable value for debug logging.
pub fn debug_body(data: &str) -> String {
    let raw = if let Ok(parsed) = serde_json::from_str::<Value>(data) {
        serde_json::to_string(&parsed).unwrap_or_else(|_| data.to_string())
    } else {
        data.to_string()
    };
    let s = redact_secrets(&raw);
    if s.len() <= DEBUG_MSG_LIMIT {
        return s;
    }
    format!("{}... ({} chars)", &s[..DEBUG_MSG_LIMIT], s.len())
}

// =============================================================================
// ERROR EXTRACTION
// =============================================================================

/// Get the error message from any error type.
fn error_message(err: &dyn std::error::Error) -> String {
    err.to_string()
}

/// Extract a descriptive error message from an axios error (or any error).
/// For HTTP errors, appends the server's response body message if available.
pub fn describe_axios_error(err: &serde_json::Value) -> String {
    let msg = if let Some(err_str) = err.get("message").and_then(|v| v.as_str()) {
        err_str.to_string()
    } else {
        "Unknown error".to_string()
    };

    if let Some(response) = err.get("response").and_then(|v| v.as_object()) {
        if let Some(data) = response.get("data").and_then(|v| v.as_object()) {
            let detail = data.get("message").and_then(|v| v.as_str()).or_else(|| {
                data.get("error")
                    .and_then(|v| v.get("message"))
                    .and_then(|v| v.as_str())
            });

            if let Some(detail) = detail {
                return format!("{}: {}", msg, detail);
            }
        }
    }
    msg
}

/// Extract the HTTP status code from an axios error, if present.
/// Returns None for non-HTTP errors (e.g. network failures).
pub fn extract_http_status(err: &serde_json::Value) -> Option<u16> {
    let response = err.get("response")?;
    let status = response.get("status")?;
    status.as_u64().map(|v| v as u16)
}

/// Pull a human-readable message out of an API error response body.
/// Checks `data.message` first, then `data.error.message`.
pub fn extract_error_detail(data: &serde_json::Value) -> Option<String> {
    if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
        return Some(msg.to_string());
    }
    if let Some(error) = data.get("error").and_then(|v| v.as_object()) {
        if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
    }
    None
}

// =============================================================================
// ANALYTICS (STUB)
// =============================================================================

/// Log a bridge init skip — debug message + `tengu_bridge_repl_skipped`
/// analytics event.
pub fn log_bridge_skip(reason: &str, debug_msg: Option<&str>, v2: Option<bool>) {
    if let Some(msg) = debug_msg {
        eprintln!("[bridge:debug] {}", msg);
    }
    // Analytics event would be logged here
    let mut event = serde_json::json!({ "reason": reason });
    if let Some(v2_val) = v2 {
        event["v2"] = serde_json::json!(v2_val);
    }
    eprintln!("[bridge:analytics] tengu_bridge_repl_skipped: {}", event);
}
