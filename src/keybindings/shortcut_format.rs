//! Shortcut format utilities

pub fn format_shortcut(key: &str) -> String {
    key.replace("ctrl", "⌃")
        .replace("alt", "⌥")
        .replace("shift", "⇧")
        .replace("meta", "⌘")
}
