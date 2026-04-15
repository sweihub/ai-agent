//! User prompt keyword matching utilities.
//!
//! Checks if input matches negative or continuation keyword patterns.

use regex::Regex;

/// Checks if input matches negative keyword patterns
pub fn matches_negative_keyword(input: &str) -> bool {
    let lower_input = input.to_lowercase();

    let negative_pattern = Regex::new(
        r"(?i)\b(wtf|wth|ffs|omfg|shit(ty|tiest)?|dumbass|horrible|awful|piss(ed|ing)? off|piece of (shit|crap|junk)|what the (fuck|hell)|fucking? (broken|useless|terrible|awful|horrible)|fuck you|screw (this|you)|so frustrating|this sucks|damn it)\b"
    ).unwrap();

    negative_pattern.is_match(&lower_input)
}

/// Checks if input matches keep going/continuation patterns
pub fn matches_keep_going_keyword(input: &str) -> bool {
    let lower_input = input.to_lowercase().trim();

    // Match "continue" only if it's the entire prompt
    if lower_input == "continue" {
        return true;
    }

    // Match "keep going" or "go on" anywhere in the input
    let keep_going_pattern = Regex::new(r"(?i)\b(keep going|go on)\b").unwrap();
    keep_going_pattern.is_match(lower_input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negative_keywords() {
        assert!(matches_negative_keyword("this is shit"));
        assert!(matches_negative_keyword("WTF is going on"));
        assert!(!matches_negative_keyword("hello world"));
    }

    #[test]
    fn test_keep_going() {
        assert!(matches_keep_going_keyword("continue"));
        assert!(matches_keep_going_keyword("keep going"));
        assert!(matches_keep_going_keyword("go on please"));
        assert!(!matches_keep_going_keyword("continue working later"));
    }
}
