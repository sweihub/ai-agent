//! Webhook payload sanitizer.
//!
//! Translated from openclaudecode/src/bridge/webhookSanitizer.ts

/// Sanitize a webhook payload value.
/// Currently a no-op, but provides a hook for future sanitization logic.
pub fn sanitize_webhook_payload<T>(value: T) -> T {
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_webhook_payload() {
        let value = serde_json::json!({"key": "value"});
        let sanitized = sanitize_webhook_payload(value);
        assert_eq!(sanitized.get("key").unwrap(), "value");
    }
}
