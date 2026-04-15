//! Semantic boolean utilities for interpreting various truthy/falsy values.

/// Convert a semantic value to boolean
pub fn to_bool(value: &str) -> bool {
    let lower = value.trim().to_lowercase();

    // Truthy values
    if lower == "true" || lower == "yes" || lower == "1" || lower == "on" {
        return true;
    }

    // Falsy values
    if lower == "false" || lower == "no" || lower == "0" || lower == "off" || lower == "none" {
        return false;
    }

    // Default to false for unknown values
    false
}

/// Check if a value is truthy (not empty, not "false", etc.)
pub fn is_truthy(value: &str) -> bool {
    !value.trim().is_empty() && to_bool(value)
}

/// Check if a value is falsy (empty, "false", "no", etc.)
pub fn is_falsy(value: &str) -> bool {
    value.trim().is_empty() || !to_bool(value)
}

/// Parse a boolean from environment variable
pub fn parse_env_bool(key: &str) -> Option<bool> {
    std::env::var(key).ok().map(|v| to_bool(&v))
}

/// Check if an environment variable is truthy
pub fn is_env_truthy(key: &str) -> bool {
    parse_env_bool(key).unwrap_or(false)
}
