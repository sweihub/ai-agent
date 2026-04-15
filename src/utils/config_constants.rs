// Source: ~/claudecode/openclaudecode/src/utils/configConstants.ts
//! Configuration constants for notification channels, editor modes, and teammate modes.
//! These constants are in a separate file to avoid circular dependency issues.
//! Do NOT add imports to this file - it must remain dependency-free.

#![allow(dead_code)]

/// Valid notification channels.
pub const NOTIFICATION_CHANNELS: &[&str] = &[
    "auto",
    "iterm2",
    "iterm2_with_bell",
    "terminal_bell",
    "kitty",
    "ghostty",
    "notifications_disabled",
];

/// Valid editor modes (excludes deprecated 'emacs' which is auto-migrated to 'normal').
pub const EDITOR_MODES: &[&str] = &["normal", "vim"];

/// Valid teammate modes for spawning.
/// 'tmux' = traditional tmux-based teammates
/// 'in-process' = in-process teammates running in same process
/// 'auto' = automatically choose based context (default)
pub const TEAMMATE_MODES: &[&str] = &["auto", "tmux", "in-process"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_channels() {
        assert!(NOTIFICATION_CHANNELS.contains(&"auto"));
        assert!(NOTIFICATION_CHANNELS.contains(&"iterm2"));
        assert!(NOTIFICATION_CHANNELS.contains(&"notifications_disabled"));
        assert_eq!(NOTIFICATION_CHANNELS.len(), 7);
    }

    #[test]
    fn test_editor_modes() {
        assert!(EDITOR_MODES.contains(&"normal"));
        assert!(EDITOR_MODES.contains(&"vim"));
        assert_eq!(EDITOR_MODES.len(), 2);
    }

    #[test]
    fn test_teammate_modes() {
        assert!(TEAMMATE_MODES.contains(&"auto"));
        assert!(TEAMMATE_MODES.contains(&"tmux"));
        assert!(TEAMMATE_MODES.contains(&"in-process"));
        assert_eq!(TEAMMATE_MODES.len(), 3);
    }
}
