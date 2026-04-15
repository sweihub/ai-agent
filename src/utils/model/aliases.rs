// Source: ~/claudecode/openclaudecode/src/utils/model/aliases.ts

/// Model alias constants.
pub const MODEL_ALIASES: &[&str] = &[
    "sonnet",
    "opus",
    "haiku",
    "best",
    "sonnet[1m]",
    "opus[1m]",
    "opusplan",
];

/// Check if a model input string is a known model alias.
pub fn is_model_alias(model_input: &str) -> bool {
    MODEL_ALIASES.contains(&model_input)
}

/// Bare model family aliases that act as wildcards in the availableModels allowlist.
/// When "opus" is in the allowlist, ANY opus model is allowed (opus 4.5, 4.6, etc.).
/// When a specific model ID is in the allowlist, only that exact version is allowed.
pub const MODEL_FAMILY_ALIASES: &[&str] = &["sonnet", "opus", "haiku"];

/// Check if a model string is a model family alias.
pub fn is_model_family_alias(model: &str) -> bool {
    MODEL_FAMILY_ALIASES.contains(&model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_model_alias() {
        assert!(is_model_alias("sonnet"));
        assert!(is_model_alias("opus[1m]"));
        assert!(!is_model_alias("claude-4-sonnet-20250514"));
    }

    #[test]
    fn test_is_model_family_alias() {
        assert!(is_model_family_alias("sonnet"));
        assert!(is_model_family_alias("opus"));
        assert!(is_model_family_alias("haiku"));
        assert!(!is_model_family_alias("best"));
    }
}
