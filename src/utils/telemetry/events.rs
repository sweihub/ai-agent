// Source: ~/claudecode/openclaudecode/src/utils/telemetry/events.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonically increasing counter for ordering events within a session.
static EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Track whether we've already warned about a null event logger to avoid spamming.
static HAS_WARNED_NO_EVENT_LOGGER: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn is_user_prompt_logging_enabled() -> bool {
    // Localized: OTEL_LOG_USER_PROMPTS
    std::env::var("OTEL_LOG_USER_PROMPTS")
        .ok()
        .map(|v| {
            let v = v.to_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

/// Redact content if user prompt logging is disabled.
pub fn redact_if_disabled(content: &str) -> String {
    if is_user_prompt_logging_enabled() {
        content.to_string()
    } else {
        "<REDACTED>".to_string()
    }
}

/// Log an OpenTelemetry event.
pub async fn log_otel_event(event_name: &str, metadata: Option<HashMap<String, String>>) {
    // Skip logging in test environment
    if std::env::var("RUST_ENV").ok().as_deref() == Some("test") {
        return;
    }

    let metadata = metadata.unwrap_or_default();
    let sequence = EVENT_SEQUENCE.fetch_add(1, Ordering::SeqCst);

    // Build attributes
    let mut attributes = get_telemetry_attributes();
    attributes.insert("event.name".to_string(), event_name.to_string());
    attributes.insert(
        "event.timestamp".to_string(),
        chrono::Utc::now().to_rfc3339(),
    );
    attributes.insert("event.sequence".to_string(), sequence.to_string());

    // Add prompt ID if available
    if let Some(prompt_id) = get_prompt_id() {
        attributes.insert("prompt.id".to_string(), prompt_id);
    }

    // Add workspace directory if available (localized: AI_CODE_*)
    if let Ok(workspace_dir) = std::env::var("AI_CODE_WORKSPACE_HOST_PATHS") {
        attributes.insert(
            "workspace.host_paths".to_string(),
            workspace_dir.replace('|', ","),
        );
    }

    // Add metadata as attributes
    for (key, value) in metadata {
        attributes.insert(key, value);
    }

    // In production, this would emit to the actual OTel event logger
    // For now, log via tracing
    tracing::debug!(
        event = event_name,
        ?attributes,
        "claude_code.{}",
        event_name
    );
}

fn get_telemetry_attributes() -> HashMap<String, String> {
    crate::utils::telemetry_attributes::get_telemetry_attributes()
}

fn get_prompt_id() -> Option<String> {
    std::env::var("AI_CODE_PROMPT_ID").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_if_disabled() {
        std::env::remove_var("OTEL_LOG_USER_PROMPTS");
        assert_eq!(redact_if_disabled("secret"), "<REDACTED>");
    }

    #[test]
    fn test_redact_if_enabled() {
        std::env::set_var("OTEL_LOG_USER_PROMPTS", "1");
        assert_eq!(redact_if_disabled("visible"), "visible");
        std::env::remove_var("OTEL_LOG_USER_PROMPTS");
    }
}
