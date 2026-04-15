// Source: ~/claudecode/openclaudecode/src/utils/permissions/bashClassifier.ts
#![allow(dead_code)]

//! Stub for external builds - classifier permissions feature is ANT-ONLY.

pub const PROMPT_PREFIX: &str = "prompt:";

/// Classifier result.
#[derive(Debug, Clone)]
pub struct ClassifierResult {
    pub matches: bool,
    pub matched_description: Option<String>,
    pub confidence: ClassifierConfidence,
    pub reason: String,
}

/// Classifier confidence level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassifierConfidence {
    High,
    Medium,
    Low,
}

/// Classifier behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassifierBehavior {
    Deny,
    Ask,
    Allow,
}

pub fn extract_prompt_description(_rule_content: Option<&str>) -> Option<String> {
    None
}

pub fn create_prompt_rule_content(description: &str) -> String {
    format!("{} {}", PROMPT_PREFIX, description.trim())
}

pub fn is_classifier_permissions_enabled() -> bool {
    false
}

pub fn get_bash_prompt_deny_descriptions(_context: &()) -> Vec<String> {
    vec![]
}

pub fn get_bash_prompt_ask_descriptions(_context: &()) -> Vec<String> {
    vec![]
}

pub fn get_bash_prompt_allow_descriptions(_context: &()) -> Vec<String> {
    vec![]
}

pub async fn classify_bash_command(
    _command: &str,
    _cwd: &str,
    _descriptions: &[String],
    _behavior: &ClassifierBehavior,
    _signal: &tokio::sync::oneshot::Receiver<()>,
    _is_non_interactive_session: bool,
) -> ClassifierResult {
    ClassifierResult {
        matches: false,
        matched_description: None,
        confidence: ClassifierConfidence::High,
        reason: "This feature is disabled".to_string(),
    }
}

pub async fn generate_generic_description(
    _command: &str,
    specific_description: Option<String>,
    _signal: &tokio::sync::oneshot::Receiver<()>,
) -> Option<String> {
    specific_description
}
