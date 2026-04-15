// Source: ~/claudecode/openclaudecode/src/utils/uuid.rs

use regex::Regex;
use std::sync::LazyLock;
use uuid::Uuid;

static UUID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()
});

/// Validate UUID format.
pub fn validate_uuid(maybe_uuid: &str) -> Option<Uuid> {
    // UUID format: 8-4-4-4-12 hex digits
    if UUID_REGEX.is_match(maybe_uuid) {
        Uuid::parse_str(maybe_uuid).ok()
    } else {
        None
    }
}

/// Generate a new agent ID with prefix for consistency with task IDs.
/// Format: a{label-}{16 hex chars}
/// Example: aa3f2c1b4d5e6f7a8, acompact-a3f2c1b4d5e6f7a8
pub fn create_agent_id(label: Option<&str>) -> String {
    let suffix = Uuid::new_v4().simple().to_string()[..16].to_string();
    match label {
        Some(l) => format!("a{l}-{suffix}"),
        None => format!("a{suffix}"),
    }
}

/// Generate a new UUID v4 string.
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_uuid_valid() {
        let valid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(validate_uuid(valid).is_some());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        assert!(validate_uuid("not-a-uuid").is_none());
        assert!(validate_uuid("").is_none());
        assert!(validate_uuid("550e8400-e29b-41d4-a716").is_none());
    }

    #[test]
    fn test_create_agent_id_no_label() {
        let id = create_agent_id(None);
        assert!(id.starts_with('a'));
        assert_eq!(id.len(), 17); // 'a' + 16 hex chars
    }

    #[test]
    fn test_create_agent_id_with_label() {
        let id = create_agent_id(Some("compact"));
        assert!(id.starts_with("acompact-"));
    }

    #[test]
    fn test_generate_uuid() {
        let uuid1 = generate_uuid();
        let uuid2 = generate_uuid();
        assert_ne!(uuid1, uuid2);
        assert!(validate_uuid(&uuid1).is_some());
    }
}
