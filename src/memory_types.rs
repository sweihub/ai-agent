pub const MEMORY_TYPES: &[&str] = &["user", "feedback", "project", "reference"];

pub fn parse_memory_type(raw: &str) -> Option<&'static str> {
    MEMORY_TYPES.iter().find(|&&t| t == raw).copied()
}
