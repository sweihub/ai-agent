// Source: /data/home/swei/claudecode/openclaudecode/src/utils/uuid.ts
use uuid::Uuid;

const UUID_REGEX: &str = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

pub type AgentId = String;

fn validate_uuid_regex(maybe_uuid: &str) -> bool {
    let regex = regex_lite::Regex::new(UUID_REGEX).unwrap();
    regex.is_match(maybe_uuid)
}

pub fn validate_uuid(maybe_uuid: &str) -> Option<Uuid> {
    if !validate_uuid_regex(maybe_uuid) {
        return None;
    }
    Uuid::parse_str(maybe_uuid).ok()
}

pub fn validate_uuid_string(maybe_uuid: &str) -> Option<String> {
    if validate_uuid_regex(maybe_uuid) {
        Some(maybe_uuid.to_string())
    } else {
        None
    }
}

pub fn create_agent_id(label: Option<&str>) -> AgentId {
    let suffix = Uuid::new_v4().simple().to_string()[..16].to_string();
    match label {
        Some(l) => format!("a{}-{}", l, suffix),
        None => format!("a{}", suffix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_uuid_valid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(validate_uuid(valid_uuid).is_some());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        let invalid_uuid = "not-a-uuid";
        assert!(validate_uuid(invalid_uuid).is_none());
    }

    #[test]
    fn test_create_agent_id() {
        let id = create_agent_id(None);
        assert!(id.starts_with('a'));
        assert_eq!(id.len(), 17);

        let id_with_label = create_agent_id(Some("test"));
        assert!(id_with_label.starts_with("at"));
        assert!(id_with_label.contains('-'));
    }
}
