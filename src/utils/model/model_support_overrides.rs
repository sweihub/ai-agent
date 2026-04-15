// Source: ~/claudecode/openclaudecode/src/utils/model/modelSupportOverrides.rs

use std::collections::HashMap;
use std::sync::LazyLock;

/// Model capability override types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelCapabilityOverride {
    Effort,
    MaxEffort,
    Thinking,
    AdaptiveThinking,
    InterleavedThinking,
}

impl ModelCapabilityOverride {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Effort => "effort",
            Self::MaxEffort => "max_effort",
            Self::Thinking => "thinking",
            Self::AdaptiveThinking => "adaptive_thinking",
            Self::InterleavedThinking => "interleaved_thinking",
        }
    }
}

/// Tier configuration for model capability overrides.
struct TierConfig {
    model_env_var: &'static str,
    capabilities_env_var: &'static str,
}

const TIERS: &[TierConfig] = &[
    TierConfig {
        model_env_var: "AI_DEFAULT_OPUS_MODEL",
        capabilities_env_var: "AI_DEFAULT_OPUS_MODEL_SUPPORTED_CAPABILITIES",
    },
    TierConfig {
        model_env_var: "AI_DEFAULT_SONNET_MODEL",
        capabilities_env_var: "AI_DEFAULT_SONNET_MODEL_SUPPORTED_CAPABILITIES",
    },
    TierConfig {
        model_env_var: "AI_DEFAULT_HAIKU_MODEL",
        capabilities_env_var: "AI_DEFAULT_HAIKU_MODEL_SUPPORTED_CAPABILITIES",
    },
];

/// Get the current API provider.
fn get_api_provider() -> &'static str {
    // Localized: CLAUDE_CODE_* -> AI_CODE_*
    if std::env::var("AI_CODE_USE_BEDROCK").is_ok() {
        return "bedrock";
    }
    if std::env::var("AI_CODE_USE_VERTEX").is_ok() {
        return "vertex";
    }
    if std::env::var("AI_CODE_USE_FOUNDRY").is_ok() {
        return "foundry";
    }
    "firstParty"
}

/// Cache for 3P model capability overrides.
static CAPABILITY_CACHE: LazyLock<
    std::sync::Mutex<HashMap<(String, String), Option<bool>>>,
> = LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

/// Check whether a 3p model capability override is set for a model that matches one of
/// the pinned AI_DEFAULT_*_MODEL env vars.
pub fn get_3p_model_capability_override(
    model: &str,
    capability: ModelCapabilityOverride,
) -> Option<bool> {
    let cache_key = (model.to_lowercase(), capability.as_str().to_string());

    // Check cache first
    {
        let cache = CAPABILITY_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&cache_key) {
            return *cached;
        }
    }

    // Not first party, so check env vars
    if get_api_provider() == "firstParty" {
        let mut cache = CAPABILITY_CACHE.lock().unwrap();
        cache.insert(cache_key, None);
        return None;
    }

    let model_lower = model.to_lowercase();

    for tier in TIERS {
        if let (Ok(pinned), Ok(capabilities)) = (
            std::env::var(tier.model_env_var),
            std::env::var(tier.capabilities_env_var),
        ) {
            if model_lower == pinned.to_lowercase() {
                let result = capabilities
                    .to_lowercase()
                    .split(',')
                    .map(|s| s.trim())
                    .any(|s| s == capability.as_str());

                let mut cache = CAPABILITY_CACHE.lock().unwrap();
                cache.insert(cache_key, Some(result));
                return Some(result);
            }
        }
    }

    let mut cache = CAPABILITY_CACHE.lock().unwrap();
    cache.insert(cache_key, None);
    None
}

/// Clear the capability override cache.
pub fn clear_capability_override_cache() {
    CAPABILITY_CACHE.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_party_returns_none() {
        std::env::remove_var("AI_CODE_USE_BEDROCK");
        std::env::remove_var("AI_CODE_USE_VERTEX");
        std::env::remove_var("AI_CODE_USE_FOUNDRY");
        clear_capability_override_cache();

        assert!(
            get_3p_model_capability_override("opus", ModelCapabilityOverride::Thinking).is_none()
        );
    }
}
