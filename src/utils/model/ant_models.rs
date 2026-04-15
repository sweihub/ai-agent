// Source: ~/claudecode/openclaudecode/src/utils/model/antModels.ts

use serde::{Deserialize, Serialize};

/// Effort level for model configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// An Anthropic model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntModel {
    pub alias: String,
    pub model: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_effort_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_effort_level: Option<EffortLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper_max_tokens_limit: Option<usize>,
    /// Model defaults to adaptive thinking and rejects `thinking: { type: 'disabled' }`.
    #[serde(skip_serializing_if = "Option::is_none", rename = "alwaysOnThinking")]
    pub always_on_thinking: Option<bool>,
}

/// Configuration for model switch callout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntModelSwitchCalloutConfig {
    #[serde(skip_serializing_if = "Option::is_none", rename = "modelAlias")]
    pub model_alias: Option<String>,
    pub description: String,
    pub version: String,
}

/// Override configuration for Anthropic models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntModelOverrideConfig {
    #[serde(skip_serializing_if = "Option::is_none", rename = "defaultModel")]
    pub default_model: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "defaultModelEffortLevel"
    )]
    pub default_model_effort_level: Option<EffortLevel>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "defaultSystemPromptSuffix"
    )]
    pub default_system_prompt_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "antModels")]
    pub ant_models: Option<Vec<AntModel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_callout: Option<AntModelSwitchCalloutConfig>,
}

/// Get the Anthropic model override configuration.
/// Returns None if not an ant user or no override configured.
pub fn get_ant_model_override_config() -> Option<AntModelOverrideConfig> {
    // In Rust, we check environment variable at runtime
    let user_type = std::env::var("USER_TYPE").ok();
    if user_type.as_deref() != Some("ant") {
        return None;
    }

    // In production, this would fetch from GrowthBook or similar feature flag service
    // For now, return None as we don't have the GrowthBook integration
    None
}

/// Get the list of Anthropic models for ant users.
pub fn get_ant_models() -> Vec<AntModel> {
    let user_type = std::env::var("USER_TYPE").ok();
    if user_type.as_deref() != Some("ant") {
        return Vec::new();
    }

    get_ant_model_override_config()
        .and_then(|config| config.ant_models)
        .unwrap_or_default()
}

/// Resolve a model string to its Anthropic model config.
pub fn resolve_ant_model(model: &str) -> Option<AntModel> {
    let user_type = std::env::var("USER_TYPE").ok();
    if user_type.as_deref() != Some("ant") {
        return None;
    }

    let lower = model.to_lowercase();
    get_ant_models().into_iter().find(|m| {
        m.alias == model || m.model.to_lowercase().contains(&lower)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ant_models_non_ant_user() {
        // Should return empty vec when USER_TYPE != "ant"
        std::env::remove_var("USER_TYPE");
        assert!(get_ant_models().is_empty());
    }

    #[test]
    fn test_resolve_ant_model_non_ant_user() {
        std::env::remove_var("USER_TYPE");
        assert!(resolve_ant_model("opus").is_none());
    }
}
