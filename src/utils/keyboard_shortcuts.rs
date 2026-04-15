// Source: ~/claudecode/openclaudecode/src/utils/keyboardShortcuts.ts

use std::collections::HashMap;
use std::sync::LazyLock;

/// Special characters that macOS Option+key produces, mapped to their
/// keybinding equivalents. Used to detect Option+key shortcuts on macOS
/// terminals that don't have "Option as Meta" enabled.
static MACOS_OPTION_SPECIAL_CHARS: LazyLock<HashMap<&str, &str>> =
    LazyLock::new(|| {
        HashMap::from([
            ("†", "alt+t"), // Option+T -> thinking toggle
            ("π", "alt+p"), // Option+P -> model picker
            ("ø", "alt+o"), // Option+O -> fast mode
        ])
    });

/// Check if a character is a macOS Option special character.
pub fn is_macos_option_char(char: &str) -> bool {
    MACOS_OPTION_SPECIAL_CHARS.contains_key(char)
}

/// Get the keybinding equivalent for a macOS Option special character.
pub fn macos_option_char_mapping(char: &str) -> Option<&'static str> {
    MACOS_OPTION_SPECIAL_CHARS.get(char).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_macos_option_char() {
        assert!(is_macos_option_char("†"));
        assert!(is_macos_option_char("π"));
        assert!(is_macos_option_char("ø"));
        assert!(!is_macos_option_char("a"));
    }

    #[test]
    fn test_macos_option_char_mapping() {
        assert_eq!(macos_option_char_mapping("†"), Some("alt+t"));
        assert_eq!(macos_option_char_mapping("π"), Some("alt+p"));
        assert_eq!(macos_option_char_mapping("ø"), Some("alt+o"));
        assert_eq!(macos_option_char_mapping("a"), None);
    }
}
