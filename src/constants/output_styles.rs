//! Output styles module
//!
//! Defines output style configurations for different modes of interaction.

use std::collections::HashMap;

/// Source of the output style configuration
#[derive(Debug, Clone, PartialEq)]
pub enum OutputStyleSource {
    BuiltIn,
    Plugin,
    PolicySettings,
    UserSettings,
    ProjectSettings,
}

impl OutputStyleSource {
    pub fn as_str(&self) -> &str {
        match self {
            OutputStyleSource::BuiltIn => "built-in",
            OutputStyleSource::Plugin => "plugin",
            OutputStyleSource::PolicySettings => "policySettings",
            OutputStyleSource::UserSettings => "userSettings",
            OutputStyleSource::ProjectSettings => "projectSettings",
        }
    }
}

/// Configuration for an output style
#[derive(Debug, Clone)]
pub struct OutputStyleConfig {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub source: OutputStyleSource,
    pub keep_coding_instructions: Option<bool>,
    pub force_for_plugin: Option<bool>,
}

impl OutputStyleConfig {
    pub fn new(name: &str, description: &str, prompt: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            prompt: prompt.to_string(),
            source: OutputStyleSource::BuiltIn,
            keep_coding_instructions: None,
            force_for_plugin: None,
        }
    }

    pub fn with_keep_coding_instructions(mut self, keep: bool) -> Self {
        self.keep_coding_instructions = Some(keep);
        self
    }

    pub fn with_source(mut self, source: OutputStyleSource) -> Self {
        self.source = source;
        self
    }
}

/// Default output style name
pub const DEFAULT_OUTPUT_STYLE_NAME: &str = "default";

/// Explanatory feature prompt used in both Explanatory and Learning modes
pub const EXPLANATORY_FEATURE_PROMPT: &str = r#"## Insights
In order to encourage learning, before and after writing code, always provide brief educational explanations about implementation choices using (with backticks):
"`* Insight ─────────────────────────────────────`
[2-3 key educational points]
`─────────────────────────────────────────────────`"

These insights should be included in the conversation, not in the codebase. You should generally focus on interesting insights that are specific to the codebase or the code you just wrote, rather than general programming concepts."#;

/// Get the built-in output style configuration
pub fn get_output_style_config() -> HashMap<String, Option<OutputStyleConfig>> {
    let mut styles = HashMap::new();

    // Default style - null means use default behavior
    styles.insert(DEFAULT_OUTPUT_STYLE_NAME.to_string(), None);

    // Explanatory style
    let explanatory_prompt = format!(
        "You are an interactive CLI tool that helps users with software engineering tasks. In addition to software engineering tasks, you should provide educational insights about the codebase along the way.\n\nYou should be clear and educational, providing helpful explanations while remaining focused on the task. Balance educational content with task completion. When providing insights, you may exceed typical length constraints, but remain focused and relevant.\n\n# Explanatory Style Active\n{}",
        EXPLANATORY_FEATURE_PROMPT
    );

    styles.insert(
        "Explanatory".to_string(),
        Some(
            OutputStyleConfig::new(
                "Explanatory",
                "Claude explains its implementation choices and codebase patterns",
                &explanatory_prompt,
            )
            .with_keep_coding_instructions(true),
        ),
    );

    // Learning style
    let learning_prompt = format!(
        "You are an interactive CLI tool that helps users with software engineering tasks. In addition to software engineering tasks, you should help users learn more about the codebase through hands-on practice and educational insights.\n\nYou should be collaborative and encouraging. Balance task completion with learning by requesting user input for meaningful design decisions while handling routine implementation yourself.\n\n# Learning Style Active\n## Requesting Human Contributions\nIn order to encourage learning, ask the human to contribute 2-10 line code pieces when generating 20+ lines involving:\n- Design decisions (error handling, data structures)\n- Business logic with multiple valid approaches\n- Key algorithms or interface definitions\n\n## Insights\n{}",
        EXPLANATORY_FEATURE_PROMPT
    );

    styles.insert(
        "Learning".to_string(),
        Some(
            OutputStyleConfig::new(
                "Learning",
                "Claude pauses and asks you to write small pieces of code for hands-on practice",
                &learning_prompt,
            )
            .with_keep_coding_instructions(true),
        ),
    );

    styles
}

/// Get all built-in output style names
pub fn get_builtin_style_names() -> Vec<&'static str> {
    vec![DEFAULT_OUTPUT_STYLE_NAME, "Explanatory", "Learning"]
}

/// Check if a style name is valid
pub fn is_valid_style_name(name: &str) -> bool {
    get_builtin_style_names().contains(&name) || name == DEFAULT_OUTPUT_STYLE_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_style() {
        let styles = get_output_style_config();
        assert!(styles.get(DEFAULT_OUTPUT_STYLE_NAME).is_some());
        assert!(styles.get(DEFAULT_OUTPUT_STYLE_NAME).unwrap().is_none());
    }

    #[test]
    fn test_explanatory_style() {
        let styles = get_output_style_config();
        let explanatory = styles.get("Explanatory").unwrap();
        assert!(explanatory.is_some());
        let config = explanatory.as_ref().unwrap();
        assert_eq!(config.name, "Explanatory");
        assert!(config.keep_coding_instructions.unwrap_or(false));
    }

    #[test]
    fn test_learning_style() {
        let styles = get_output_style_config();
        let learning = styles.get("Learning").unwrap();
        assert!(learning.is_some());
        let config = learning.as_ref().unwrap();
        assert_eq!(config.name, "Learning");
    }

    #[test]
    fn test_valid_style_names() {
        assert!(is_valid_style_name("default"));
        assert!(is_valid_style_name("Explanatory"));
        assert!(is_valid_style_name("Learning"));
        assert!(!is_valid_style_name("Invalid"));
    }
}
