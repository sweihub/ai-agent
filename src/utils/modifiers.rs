// Source: ~/claudecode/openclaudecode/src/utils/modifiers.ts

use serde::{Deserialize, Serialize};

/// Modifier key types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKey {
    Shift,
    Command,
    Control,
    Option,
}

impl ModifierKey {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Shift => "shift",
            Self::Command => "command",
            Self::Control => "control",
            Self::Option => "option",
        }
    }
}

/// A keyboard modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modifier {
    Shift,
    Ctrl,
    Alt,
    Meta,
}

impl Modifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Shift => "shift",
            Self::Ctrl => "ctrl",
            Self::Alt => "alt",
            Self::Meta => "meta",
        }
    }
}

/// A keyboard shortcut consisting of optional modifiers and a key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub modifiers: Vec<Modifier>,
    pub key: String,
}

impl Shortcut {
    pub fn new(key: impl Into<String>, modifiers: Vec<Modifier>) -> Self {
        Self {
            modifiers,
            key: key.into(),
        }
    }

    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        for m in &self.modifiers {
            parts.push(match m {
                Modifier::Shift => "⇧",
                Modifier::Ctrl => "⌃",
                Modifier::Alt => "⌥",
                Modifier::Meta => "⌘",
            });
        }
        parts.push(&self.key);
        parts.join("")
    }
}

static PREWARMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Pre-warm the native module by loading it in advance.
/// Call this early to avoid delay on first use.
/// Note: This is a no-op on non-macOS platforms.
pub fn prewarm_modifiers() {
    if PREWARMED.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }

    // Only on macOS
    if !cfg!(target_os = "macos") {
        return;
    }

    PREWARMED.store(true, std::sync::atomic::Ordering::SeqCst);

    // In production, this would load the native module
    // For now, this is a no-op as we don't have the native bindings
}

/// Check if a specific modifier key is currently pressed.
/// Note: This is a no-op on non-macOS platforms.
pub fn is_modifier_pressed(_modifier: ModifierKey) -> bool {
    // Only on macOS
    if !cfg!(target_os = "macos") {
        return false;
    }

    // In production, this would call the native module
    // For now, always return false as we don't have the native bindings
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prewarm_does_not_panic() {
        prewarm_modifiers();
        prewarm_modifiers(); // Should be idempotent
    }

    #[test]
    fn test_is_modifier_pressed() {
        // On non-macOS this always returns false
        // On macOS without native module it also returns false
        let _ = is_modifier_pressed(ModifierKey::Shift);
    }
}
