// Source: ~/claudecode/openclaudecode/src/utils/telemetryAttributes.rs

use std::collections::HashMap;

/// Default configuration for metrics cardinality.
struct MetricsCardinalityDefaults {
    otel_metrics_include_session_id: bool,
    otel_metrics_include_version: bool,
    otel_metrics_include_account_uuid: bool,
}

impl Default for MetricsCardinalityDefaults {
    fn default() -> Self {
        Self {
            otel_metrics_include_session_id: true,
            otel_metrics_include_version: false,
            otel_metrics_include_account_uuid: true,
        }
    }
}

fn should_include_attribute(env_var: &str) -> bool {
    let defaults = MetricsCardinalityDefaults::default();
    let default_value = match env_var {
        "OTEL_METRICS_INCLUDE_SESSION_ID" => defaults.otel_metrics_include_session_id,
        "OTEL_METRICS_INCLUDE_VERSION" => defaults.otel_metrics_include_version,
        "OTEL_METRICS_INCLUDE_ACCOUNT_UUID" => defaults.otel_metrics_include_account_uuid,
        _ => false,
    };

    match std::env::var(env_var).ok() {
        Some(v) => is_env_truthy(&v),
        None => default_value,
    }
}

fn is_env_truthy(value: &str) -> bool {
    let v = value.to_lowercase();
    v == "1" || v == "true" || v == "yes" || v == "on"
}

/// Get telemetry attributes for metrics and events.
pub fn get_telemetry_attributes() -> HashMap<String, String> {
    let mut attributes = HashMap::new();

    let user_id = get_or_create_user_id();
    attributes.insert("user.id".to_string(), user_id);

    if should_include_attribute("OTEL_METRICS_INCLUDE_SESSION_ID") {
        let session_id = get_session_id();
        attributes.insert("session.id".to_string(), session_id);
    }

    if should_include_attribute("OTEL_METRICS_INCLUDE_VERSION") {
        if let Ok(version) = std::env::var("AI_CODE_VERSION") {
            attributes.insert("app.version".to_string(), version);
        }
    }

    // Add terminal type if available
    if let Ok(terminal) = std::env::var("TERM") {
        attributes.insert("terminal.type".to_string(), terminal);
    }

    attributes
}

fn get_or_create_user_id() -> String {
    // Check for existing user ID in config or generate one
    std::env::var("AI_CODE_USER_ID")
        .ok()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

fn get_session_id() -> String {
    std::env::var("AI_CODE_SESSION_ID")
        .ok()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_env_truthy() {
        assert!(is_env_truthy("1"));
        assert!(is_env_truthy("true"));
        assert!(is_env_truthy("TRUE"));
        assert!(!is_env_truthy("0"));
        assert!(!is_env_truthy("false"));
    }

    #[test]
    fn test_get_telemetry_attributes_has_user_id() {
        let attrs = get_telemetry_attributes();
        assert!(attrs.contains_key("user.id"));
    }
}
