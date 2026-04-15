// Source: /data/home/swei/claudecode/openclaudecode/src/utils/thinking.ts
//! Thinking configuration and utilities
//! Translated from /data/home/swei/claudecode/openclaudecode/src/utils/config.ts (thinking section)
//! and /data/home/swei/claudecode/openclaudecode/src/utils/thinking.ts

use std::env;

/// Thinking configuration types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ThinkingConfig {
    Adaptive,
    Enabled { budget_tokens: u32 },
    Disabled,
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        ThinkingConfig::Disabled
    }
}

/// Check if text contains the "ultrathink" keyword (case insensitive, word boundary)
pub fn has_ultrathink_keyword(text: &str) -> bool {
    regex::Regex::new(r"(?i)\bultrathink\b")
        .unwrap()
        .is_match(text)
}

/// Find positions of "ultrathink" keyword in text for UI highlighting/notification
/// Returns a vector of (word, start, end) tuples
pub fn find_thinking_trigger_positions(text: &str) -> Vec<(String, usize, usize)> {
    let mut positions = Vec::new();
    // Use fresh /g regex each call — shared regex would leak lastIndex state
    let re = regex::Regex::new(r"(?i)\bultrathink\b").unwrap();
    for cap in re.find_iter(text) {
        positions.push((cap.as_str().to_string(), cap.start(), cap.end()));
    }
    positions
}

#[cfg(feature = "theme")]
use crate::types::Theme;

/// Get rainbow color for a character index
/// Note: Theme color types need to be defined in the SDK
#[cfg(feature = "theme")]
pub fn get_rainbow_color(char_index: usize, shimmer: bool) -> &'static str {
    const RAINBOW_COLORS: &[&str] = &[
        "rainbow_red",
        "rainbow_orange",
        "rainbow_yellow",
        "rainbow_green",
        "rainbow_blue",
        "rainbow_indigo",
        "rainbow_violet",
    ];

    const RAINBOW_SHIMMER_COLORS: &[&str] = &[
        "rainbow_red_shimmer",
        "rainbow_orange_shimmer",
        "rainbow_yellow_shimmer",
        "rainbow_green_shimmer",
        "rainbow_blue_shimmer",
        "rainbow_indigo_shimmer",
        "rainbow_violet_shimmer",
    ];

    let colors = if shimmer {
        RAINBOW_SHIMMER_COLORS
    } else {
        RAINBOW_COLORS
    };
    colors[char_index % colors.len()]
}

/// Check if ultrathink is enabled
/// This checks the build-time feature flag and runtime GrowthBook flag
/// Note: In Rust, we use environment variable AI_ULTRATHINK instead of bun:bundle feature
pub fn is_ultrathink_enabled() -> bool {
    // Check environment variable (AI_ prefix for localization)
    // feature('ULTRATHINK') in TS — we check the env var as the runtime gate
    if !crate::utils::env_utils::is_env_truthy(
        std::env::var("AI_ULTRATHINK").ok().as_deref(),
    ) && !crate::utils::env_utils::is_env_truthy(
        std::env::var("ULTRATHINK").ok().as_deref(),
    ) {
        return false;
    }
    // GrowthBook feature check would go here if we have analytics integrated
    // For now, we just check the env var
    true
}

/// Provider-aware thinking support detection
/// Note: This references getCanonicalName and getAPIProvider which need model utilities
pub fn model_supports_thinking(model: &str) -> bool {
    // TODO: Integrate with model capability overrides
    // TODO: Check USER_TYPE environment variable
    // TODO: Implement getCanonicalName and getAPIProvider

    let model_lower = model.to_lowercase();

    // Anthropic models: all Claude 4+ support thinking
    if model_lower.contains("claude") {
        // Check for Claude 3 models (don't support thinking)
        if model_lower.contains("claude-3-") || model_lower.contains("claude 3") {
            return false;
        }
        // Claude 4+ supports thinking
        return true;
    }

    // Other providers: check for specific model versions
    // Sonnet 4+ and Opus 4+ support thinking
    model_lower.contains("sonnet-4") || model_lower.contains("opus-4")
}

/// Check if model supports adaptive thinking
/// Adaptive thinking is supported by a subset of Claude 4 models
pub fn model_supports_adaptive_thinking(model: &str) -> bool {
    let canonical = model.to_lowercase();

    // Supported by opus-4-6 and sonnet-4-6 variants
    if canonical.contains("opus-4-6") || canonical.contains("sonnet-4-6") {
        return true;
    }

    // Exclude legacy models
    if canonical.contains("opus") || canonical.contains("sonnet") || canonical.contains("haiku") {
        return false;
    }

    // Default to true for unknown models on first-party and foundry
    // (these are proxies that should support adaptive thinking)
    // TODO: Check getAPIProvider() - need to implement
    true
}

/// Check if thinking should be enabled by default
pub fn should_enable_thinking_by_default() -> bool {
    // Check MAX_THINKING_TOKENS environment variable
    if let Ok(val) = env::var("MAX_THINKING_TOKENS") {
        if let Ok(tokens) = val.parse::<u32>() {
            return tokens > 0;
        }
    }

    // Check AI_MAX_THINKING_TOKENS (localized env var)
    if let Ok(val) = env::var("AI_MAX_THINKING_TOKENS") {
        if let Ok(tokens) = val.parse::<u32>() {
            return tokens > 0;
        }
    }

    // Check settings for alwaysThinkingEnabled
    // TODO: Integrate with settings system
    // For now, default to enabled
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_ultrathink_keyword() {
        assert!(has_ultrathink_keyword("Let's ultrathink this problem"));
        assert!(has_ultrathink_keyword("ULTRATHINK"));
        assert!(has_ultrathink_keyword("UltraThink"));
        assert!(!has_ultrathink_keyword("thinking is good"));
    }

    #[test]
    fn test_find_thinking_trigger_positions() {
        let text = "Use ultrathink for this. Also ULTRATHINK again.";
        let positions = find_thinking_trigger_positions(text);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].0, "ultrathink");
        assert_eq!(positions[1].0, "ULTRATHINK");
    }

    #[test]
    fn test_model_supports_thinking() {
        assert!(model_supports_thinking("claude-opus-4-20250514"));
        assert!(model_supports_thinking("claude-sonnet-4-20250514"));
        assert!(!model_supports_thinking("claude-3-5-sonnet-20241022"));
    }

    #[test]
    fn test_model_supports_adaptive_thinking() {
        assert!(model_supports_adaptive_thinking("opus-4-6-20250514"));
        assert!(model_supports_adaptive_thinking("sonnet-4-6-20250514"));
        assert!(!model_supports_adaptive_thinking("claude-3-5-sonnet"));
    }

    #[test]
    fn test_should_enable_thinking_by_default() {
        // Without env var, should default to true
        assert!(should_enable_thinking_by_default());
    }
}
