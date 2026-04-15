#![allow(dead_code)]

use std::collections::HashMap;

static EVENT_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static HAS_WARNED_NO_EVENT_LOGGER: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn redact_if_disabled(content: &str, _user_prompt_logging_enabled: bool) -> String {
    if _user_prompt_logging_enabled {
        content.to_string()
    } else {
        "<REDACTED>".to_string()
    }
}

pub async fn log_otel_event(
    event_name: &str,
    metadata: HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
