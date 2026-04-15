// Source: ~/claudecode/openclaudecode/src/utils/ultraplan/prompt.txt

/// Ultraplan is unavailable in the restored development build.
pub const ULTRAPLAN_PROMPT: &str = "Ultraplan is unavailable in the restored development build.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ultraplan_prompt() {
        assert_eq!(
            ULTRAPLAN_PROMPT,
            "Ultraplan is unavailable in the restored development build."
        );
    }
}
