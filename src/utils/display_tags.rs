// Source: ~/claudecode/openclaudecode/src/utils/displayTags.ts
//! Matches any XML-like `<tag>…</tag>` block (lowercase tag names, optional
//! attributes, multi-line content). Used to strip system-injected wrapper tags
//! from display titles — IDE context, slash-command markers, hook output,
//! task notifications, channel messages, etc. A generic pattern avoids
//! maintaining an ever-growing allowlist that falls behind as new notification
//! types are added.
//!
//! Only matches lowercase tag names (`[a-z][\w-]*`) so user prose mentioning
//! JSX/HTML components ("fix the <Button> layout", "<!DOCTYPE html>") passes
//! through — those start with uppercase or `!`. The non-greedy body with a
//! backreferenced closing tag keeps adjacent blocks separate; unpaired angle
//! brackets ("when x < y") don't match.

#![allow(dead_code)]

use once_cell::sync::Lazy;
use regex::Regex;

/// Matches any XML-like `<tag>…</tag>` block with lowercase tag names.
static XML_TAG_BLOCK_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<([a-z][\w-]*)(?:\s[^>]*)?>[\s\S]*?</\1>\n?").unwrap()
});

/// Matches only IDE-injected context tags.
static IDE_CONTEXT_TAGS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<(ide_opened_file|ide_selection)(?:\s[^>]*)?>[\s\S]*?</\1>\n?").unwrap()
});

/// Strip XML-like tag blocks from text for use in UI titles (/rewind, /resume,
/// bridge session titles). System-injected context — IDE metadata, hook output,
/// task notifications — arrives wrapped in tags and should never surface as a
/// title.
///
/// If stripping would result in empty text, returns the original unchanged
/// (better to show something than nothing).
pub fn strip_display_tags(text: &str) -> String {
    let result = XML_TAG_BLOCK_PATTERN.replace_all(text, "").trim().to_string();
    if result.is_empty() {
        text.to_string()
    } else {
        result
    }
}

/// Like strip_display_tags but returns empty string when all content is tags.
/// Used by get_log_display_title to detect command-only prompts (e.g. /clear)
/// so they can fall through to the next title fallback, and by extract_title_text
/// to skip pure-XML messages during bridge title derivation.
pub fn strip_display_tags_allow_empty(text: &str) -> String {
    XML_TAG_BLOCK_PATTERN
        .replace_all(text, "")
        .trim()
        .to_string()
}

/// Strip only IDE-injected context tags (ide_opened_file, ide_selection).
/// Used by text_for_resubmit so UP-arrow resubmit preserves user-typed content
/// including lowercase HTML like `<code>foo</code>` while dropping IDE noise.
pub fn strip_ide_context_tags(text: &str) -> String {
    IDE_CONTEXT_TAGS_PATTERN
        .replace_all(text, "")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_display_tags() {
        assert_eq!(strip_display_tags("<foo>bar</foo>"), "bar");
        assert_eq!(
            strip_display_tags("hello <foo>bar</foo> world"),
            "hello  world"
        );
        assert_eq!(strip_display_tags("plain text"), "plain text");
    }

    #[test]
    fn test_strip_display_tags_empty_result() {
        // If result is empty, return original
        assert_eq!(strip_display_tags("<foo></foo>"), "<foo></foo>");
    }

    #[test]
    fn test_strip_display_tags_allow_empty() {
        assert_eq!(strip_display_tags_allow_empty("<foo>bar</foo>"), "");
        assert_eq!(strip_display_tags_allow_empty("plain text"), "plain text");
    }

    #[test]
    fn test_strip_ide_context_tags() {
        assert_eq!(
            strip_ide_context_tags("<ide_opened_file>test</ide_opened_file>"),
            ""
        );
        // Should NOT strip <code> because it's not an IDE context tag
        assert_eq!(
            strip_ide_context_tags("<code>foo</code>"),
            "<code>foo</code>"
        );
    }

    #[test]
    fn test_uppercase_tags_not_stripped() {
        // Uppercase tags should pass through (JSX/HTML components)
        assert_eq!(
            strip_display_tags("fix the <Button> layout"),
            "fix the <Button> layout"
        );
        assert_eq!(
            strip_display_tags("<!DOCTYPE html>"),
            "<!DOCTYPE html>"
        );
    }

    #[test]
    fn test_unpaired_angle_brackets() {
        // Unpaired angle brackets should not match
        assert_eq!(strip_display_tags("when x < y"), "when x < y");
    }
}
