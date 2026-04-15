//! Model validation functions.
//!
//! Translated from openclaudecode/src/utils/model/validateModel.ts

use crate::constants::env::ai;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// =============================================================================
// TYPES
// =============================================================================

/// Model validation result
#[derive(Debug, Clone)]
pub struct ModelValidationResult {
    /// Whether the model is valid
    pub valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
}

impl ModelValidationResult {
    pub fn valid() -> Self {
        Self {
            valid: true,
            error: None,
        }
    }

    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            error: Some(error.into()),
        }
    }
}

// =============================================================================
// CACHE
// =============================================================================

/// Cache valid models to avoid repeated API calls
static VALID_MODEL_CACHE: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();

fn get_valid_model_cache() -> &'static Mutex<HashMap<String, bool>> {
    VALID_MODEL_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_valid_model(model: &str) {
    let mut cache = get_valid_model_cache().lock().unwrap();
    cache.insert(model.to_string(), true);
}

fn is_cached_as_valid(model: &str) -> bool {
    let cache = get_valid_model_cache().lock().unwrap();
    cache.get(model).copied().unwrap_or(false)
}

// =============================================================================
// MODEL ALIASES
// =============================================================================

/// Model aliases
const MODEL_ALIASES: &[&str] = &["opus", "sonnet", "haiku", "opusplan", "haikuplan", "best"];

fn is_model_alias(model: &str) -> bool {
    MODEL_ALIASES.contains(&model.to_lowercase().as_str())
}

// =============================================================================
// VALIDATION
// =============================================================================

/// Validates a model by attempting an actual API call.
/// This is an async function that would make an actual API call.
/// In this stub, we provide a simplified synchronous version.
pub async fn validate_model(model: &str) -> ModelValidationResult {
    let normalized_model = model.trim().to_string();

    // Empty model is invalid
    if normalized_model.is_empty() {
        return ModelValidationResult::invalid("Model name cannot be empty");
    }

    // Check against availableModels allowlist before any API call
    if !is_model_allowed(&normalized_model) {
        return ModelValidationResult::invalid(format!(
            "Model '{}' is not in the list of available models",
            normalized_model
        ));
    }

    // Check if it's a known alias (these are always valid)
    let lower_model = normalized_model.to_lowercase();
    if MODEL_ALIASES.contains(&lower_model.as_str()) {
        return ModelValidationResult::valid();
    }

    // Check if it matches ANTHROPIC_CUSTOM_MODEL_OPTION (pre-validated by the user)
    if let Ok(custom_model) = std::env::var(ai::ANTHROPIC_CUSTOM_MODEL_OPTION) {
        if normalized_model == custom_model {
            return ModelValidationResult::valid();
        }
    }

    // Check cache first
    if is_cached_as_valid(&normalized_model) {
        return ModelValidationResult::valid();
    }

    // Try to make an actual API call with minimal parameters
    // In a real implementation, this would call sideQuery or similar
    match do_validate_api_call(&normalized_model).await {
        Ok(_) => {
            // If we got here, the model is valid
            cache_valid_model(&normalized_model);
            ModelValidationResult::valid()
        }
        Err(e) => handle_validation_error(e, &normalized_model),
    }
}

/// Do actual API call to validate model
async fn do_validate_api_call(_model: &str) -> Result<(), ValidationError> {
    // Stub - would need to make actual API call via sideQuery or similar
    // For now, we'll just return Ok as a placeholder
    // In the real implementation, this would call the API and return an error if it fails
    Ok(())
}

/// Validation error types
#[derive(Debug)]
pub enum ValidationError {
    NotFound(String),
    Authentication(String),
    Connection(String),
    Api(String),
    Unknown(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            ValidationError::Authentication(msg) => write!(f, "Authentication: {}", msg),
            ValidationError::Connection(msg) => write!(f, "Connection: {}", msg),
            ValidationError::Api(msg) => write!(f, "Api: {}", msg),
            ValidationError::Unknown(msg) => write!(f, "Unknown: {}", msg),
        }
    }
}

/// Handle validation error and return appropriate result
fn handle_validation_error(error: ValidationError, model_name: &str) -> ModelValidationResult {
    match error {
        // NotFoundError (404) means the model doesn't exist
        ValidationError::NotFound(_) => {
            let fallback = get_3p_fallback_suggestion(model_name);
            let suggestion = fallback
                .map(|f| format!(". Try '{}' instead", f))
                .unwrap_or_default();
            ModelValidationResult::invalid(format!(
                "Model '{}' not found{}",
                model_name, suggestion
            ))
        }

        // For other API errors, provide context-specific messages
        ValidationError::Authentication(_) => ModelValidationResult::invalid(
            "Authentication failed. Please check your API credentials.",
        ),

        ValidationError::Connection(_) => {
            ModelValidationResult::invalid("Network error. Please check your internet connection.")
        }

        // Check error body for model-specific errors
        ValidationError::Api(msg) => {
            if msg.contains("model:") && msg.contains("not_found_error") {
                return ModelValidationResult::invalid(format!("Model '{}' not found", model_name));
            }
            // Generic API error
            ModelValidationResult::invalid(format!("API error: {}", msg))
        }

        // For unknown errors, be safe and reject
        ValidationError::Unknown(msg) => {
            ModelValidationResult::invalid(format!("Unable to validate model: {}", msg))
        }
    }
}

// =============================================================================
// FALLBACK SUGGESTIONS
// =============================================================================

/// Suggest a fallback model for 3P users when the selected model is unavailable.
fn get_3p_fallback_suggestion(model: &str) -> Option<String> {
    if get_api_provider() == "firstParty" {
        return None;
    }

    let lower_model = model.to_lowercase();

    if lower_model.contains("opus-4-6") || lower_model.contains("opus_4_6") {
        return Some(get_model_strings().opus_41.clone());
    }
    if lower_model.contains("sonnet-4-6") || lower_model.contains("sonnet_4_6") {
        return Some(get_model_strings().sonnet_45.clone());
    }
    if lower_model.contains("sonnet-4-5") || lower_model.contains("sonnet_4_5") {
        return Some(get_model_strings().sonnet_40.clone());
    }

    None
}

// =============================================================================
// STUB HELPERS
// =============================================================================

/// Get API provider
fn get_api_provider() -> String {
    std::env::var(ai::API_PROVIDER)
        .ok()
        .unwrap_or_else(|| "firstParty".to_string())
}

/// Check if model is allowed (from modelAllowlist)
fn is_model_allowed(_model: &str) -> bool {
    // Stub - would need modelAllowlist module
    // For now, allow all models
    true
}

/// Get model strings
fn get_model_strings() -> ModelStrings {
    ModelStrings {
        opus_41: "claude-opus-4-1-20250805".to_string(),
        opus_45: "claude-opus-4-5-20250514".to_string(),
        opus_46: "claude-opus-4-6-20251106".to_string(),
        sonnet_40: "claude-sonnet-4-0-20250514".to_string(),
        sonnet_45: "claude-sonnet-4-5-20241022".to_string(),
        sonnet_46: "claude-sonnet-4-6-20251106".to_string(),
    }
}

#[derive(Debug, Clone)]
struct ModelStrings {
    opus_41: String,
    opus_45: String,
    opus_46: String,
    sonnet_40: String,
    sonnet_45: String,
    sonnet_46: String,
}

impl ModelStrings {
    fn opus_41(&self) -> String {
        self.opus_41.clone()
    }
    fn opus_45(&self) -> String {
        self.opus_45.clone()
    }
    fn opus_46(&self) -> String {
        self.opus_46.clone()
    }
    fn sonnet_40(&self) -> String {
        self.sonnet_40.clone()
    }
    fn sonnet_45(&self) -> String {
        self.sonnet_45.clone()
    }
    fn sonnet_46(&self) -> String {
        self.sonnet_46.clone()
    }
}
