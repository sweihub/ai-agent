//! Reserved shortcuts

pub fn get_reserved_shortcuts() -> Vec<ReservedShortcut> {
    vec![ReservedShortcut {
        key: "ctrl+w".to_string(),
        reason: "Often used by terminal to close tab".to_string(),
        severity: "warning".to_string(),
    }]
}

pub struct ReservedShortcut {
    pub key: String,
    pub reason: String,
    pub severity: String,
}
