// Source: /data/home/swei/claudecode/openclaudecode/src/services/notifier.ts
//! Notifier service - sends system notifications.
//!
////! Translates notifier.ts from claude code.

pub const DEFAULT_TITLE: &str = "Claude Code";

#[derive(Debug, Clone)]
pub struct NotificationOptions {
    pub message: String,
    pub title: Option<String>,
    pub notification_type: String,
}

impl NotificationOptions {
    pub fn new(message: impl Into<String>, notification_type: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            title: None,
            notification_type: notification_type.into(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationChannel {
    Auto,
    Iterm2,
    Iterm2WithBell,
    Kitty,
    Ghostty,
    TerminalBell,
    NotificationsDisabled,
    None,
    Error,
    NoMethodAvailable,
}

impl NotificationChannel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "auto" => NotificationChannel::Auto,
            "iterm2" => NotificationChannel::Iterm2,
            "iterm2_with_bell" => NotificationChannel::Iterm2WithBell,
            "kitty" => NotificationChannel::Kitty,
            "ghostty" => NotificationChannel::Ghostty,
            "terminal_bell" => NotificationChannel::TerminalBell,
            "notifications_disabled" => NotificationChannel::NotificationsDisabled,
            _ => NotificationChannel::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationChannel::Auto => "auto",
            NotificationChannel::Iterm2 => "iterm2",
            NotificationChannel::Iterm2WithBell => "iterm2_with_bell",
            NotificationChannel::Kitty => "kitty",
            NotificationChannel::Ghostty => "ghostty",
            NotificationChannel::TerminalBell => "terminal_bell",
            NotificationChannel::NotificationsDisabled => "disabled",
            NotificationChannel::None => "none",
            NotificationChannel::Error => "error",
            NotificationChannel::NoMethodAvailable => "no_method_available",
        }
    }
}

pub fn generate_kitty_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    (nanos % 10000) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_options() {
        let opts = NotificationOptions::new("Test message", "test");
        assert_eq!(opts.message, "Test message");
        assert_eq!(opts.notification_type, "test");
        assert!(opts.title.is_none());
    }

    #[test]
    fn test_notification_options_with_title() {
        let opts = NotificationOptions::new("Test message", "test").with_title("Test Title");
        assert_eq!(opts.title, Some("Test Title".to_string()));
    }

    #[test]
    fn test_notification_channel_from_str() {
        assert_eq!(
            NotificationChannel::from_str("auto"),
            NotificationChannel::Auto
        );
        assert_eq!(
            NotificationChannel::from_str("iterm2"),
            NotificationChannel::Iterm2
        );
        assert_eq!(
            NotificationChannel::from_str("kitty"),
            NotificationChannel::Kitty
        );
        assert_eq!(
            NotificationChannel::from_str("unknown"),
            NotificationChannel::None
        );
    }

    #[test]
    fn test_generate_kitty_id() {
        let id = generate_kitty_id();
        assert!(id < 10000);
    }
}
