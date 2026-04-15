// Source: ~/claudecode/openclaudecode/src/utils/model/deprecation.ts

use serde::Serialize;
use std::collections::HashMap;

/// API Provider for model deprecation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiProvider {
    FirstParty,
    Bedrock,
    Vertex,
    Foundry,
}

/// Information about a deprecated model.
#[derive(Debug, Clone)]
pub struct DeprecatedModelInfo {
    pub model_name: String,
    pub retirement_date: String,
}

/// Entry for a deprecated model.
struct DeprecationEntry {
    model_name: String,
    retirement_dates: HashMap<ApiProvider, Option<String>>,
}

/// Deprecated models and their retirement dates by provider.
fn deprecated_models() -> HashMap<&'static str, DeprecationEntry> {
    HashMap::from([
        (
            "claude-3-opus",
            DeprecationEntry {
                model_name: "Claude 3 Opus".to_string(),
                retirement_dates: HashMap::from([
                    (ApiProvider::FirstParty, Some("January 5, 2026".to_string())),
                    (ApiProvider::Bedrock, Some("January 15, 2026".to_string())),
                    (ApiProvider::Vertex, Some("January 5, 2026".to_string())),
                    (ApiProvider::Foundry, Some("January 5, 2026".to_string())),
                ]),
            },
        ),
        (
            "claude-3-7-sonnet",
            DeprecationEntry {
                model_name: "Claude 3.7 Sonnet".to_string(),
                retirement_dates: HashMap::from([
                    (ApiProvider::FirstParty, Some("February 19, 2026".to_string())),
                    (ApiProvider::Bedrock, Some("April 28, 2026".to_string())),
                    (ApiProvider::Vertex, Some("May 11, 2026".to_string())),
                    (ApiProvider::Foundry, Some("February 19, 2026".to_string())),
                ]),
            },
        ),
        (
            "claude-3-5-haiku",
            DeprecationEntry {
                model_name: "Claude 3.5 Haiku".to_string(),
                retirement_dates: HashMap::from([
                    (ApiProvider::FirstParty, Some("February 19, 2026".to_string())),
                    (ApiProvider::Bedrock, None),
                    (ApiProvider::Vertex, None),
                    (ApiProvider::Foundry, None),
                ]),
            },
        ),
    ])
}

/// Check if a model is deprecated and get its deprecation info.
fn get_deprecated_model_info(model_id: &str) -> Option<DeprecatedModelInfo> {
    let lowercase_model_id = model_id.to_lowercase();
    let provider = get_api_provider();

    for (key, entry) in deprecated_models() {
        let retirement_date = entry.retirement_dates.get(&provider)?.clone()?;
        if lowercase_model_id.contains(key) {
            return Some(DeprecatedModelInfo {
                model_name: entry.model_name,
                retirement_date,
            });
        }
    }

    None
}

/// Get the current API provider.
fn get_api_provider() -> ApiProvider {
    // Check environment variables for provider
    if std::env::var("AI_CODE_USE_BEDROCK").is_ok() {
        return ApiProvider::Bedrock;
    }
    if std::env::var("AI_CODE_USE_VERTEX").is_ok() {
        return ApiProvider::Vertex;
    }
    if std::env::var("AI_CODE_USE_FOUNDRY").is_ok() {
        return ApiProvider::Foundry;
    }
    ApiProvider::FirstParty
}

/// Get a deprecation warning message for a model, or None if not deprecated.
pub fn get_model_deprecation_warning(model_id: Option<&str>) -> Option<String> {
    let model_id = model_id?;
    let info = get_deprecated_model_info(model_id)?;

    Some(format!(
        "\u{26a0} {} will be retired on {}. Consider switching to a newer model.",
        info.model_name, info.retirement_date
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_deprecation_warning_none() {
        assert!(get_model_deprecation_warning(None).is_none());
        assert!(get_model_deprecation_warning(Some("unknown-model")).is_none());
    }
}
