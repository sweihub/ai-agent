use regex::Regex;
use uuid::Uuid;

pub fn validate_uuid(maybe_uuid: &str) -> Option<String> {
    let uuid_regex =
        Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();

    if uuid_regex.is_match(maybe_uuid) {
        Some(maybe_uuid.to_string())
    } else {
        None
    }
}

pub fn create_agent_id(label: Option<&str>) -> String {
    let suffix = Uuid::new_v4().simple().to_string()[..16].to_string();
    match label {
        Some(l) => format!("a{}-{}", l, suffix),
        None => format!("a{}", suffix),
    }
}
