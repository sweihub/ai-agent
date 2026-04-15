// Source: ~/claudecode/openclaudecode/src/utils/userPromptKeywords.rs

use regex::Regex;
use std::sync::LazyLock;

static NEGATIVE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(wtf|wth|ffs|omfg|shit(ty|tiest)?|dumbass|horrible|awful|piss(ed|ing)? off|piece of (shit|crap|junk)|what the (fuck|hell)|fucking? (broken|useless|terrible|awful|horrible)|fuck you|screw (this|you)|so frustrating|this sucks|damn it)\b",
    )
    .unwrap()
});

static KEEP_GOING_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(keep going|go on)\b").unwrap());

/// Check if input matches negative keyword patterns.
pub fn matches_negative_keyword(input: &str) -> bool {
    NEGATIVE_PATTERN.is_match(input)
}

/// Check if input matches keep going/continuation patterns.
pub fn matches_keep_going_keyword(input: &str) -> bool {
    let lower_input = input.to_lowercase().trim().to_string();

    // Match "continue" only if it's the entire prompt
    if lower_input == "continue" {
        return true;
    }

    // Match "keep going" or "go on" anywhere in the input
    KEEP_GOING_PATTERN.is_match(&lower_input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negative_keywords() {
        assert!(matches_negative_keyword("wtf is this"));
        assert!(matches_negative_keyword("this sucks"));
        assert!(matches_negative_keyword("what the hell"));
        assert!(!matches_negative_keyword("this is great"));
    }

    #[test]
    fn test_keep_going_keywords() {
        assert!(matches_keep_going_keyword("continue"));
        assert!(matches_keep_going_keyword("keep going"));
        assert!(matches_keep_going_keyword("please keep going"));
        assert!(matches_keep_going_keyword("go on"));
        assert!(!matches_keep_going_keyword("stop"));
    }

    #[test]
    fn test_continue_only_exact() {
        assert!(matches_keep_going_keyword("continue"));
        assert!(!matches_keep_going_keyword("continue working"));
    }
}
