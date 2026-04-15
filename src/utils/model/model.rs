// Source: /data/home/swei/claudecode/openclaudecode/src/utils/model/model.ts
//! Model types and main loop model functions.
//!
//! Translated from openclaudecode/src/utils/model/model.ts

use crate::constants::env::{ai, ai_code};
use std::sync::OnceLock;

/// Model short name type (canonical form)
pub type ModelShortName = String;

/// Full model name type
pub type ModelName = String;

/// Model setting (can be model name, alias, or null)
pub type ModelSetting = Option<ModelNameOrAlias>;

/// Model name or alias
pub type ModelNameOrAlias = String;

/// Alias for model name
pub type ModelAlias = String;

// =============================================================================
// STUB FUNCTIONS - These need to be implemented with actual module dependencies
// =============================================================================

/// Get small fast model (Haiku)
pub fn get_small_fast_model() -> ModelName {
    std::env::var(ai::SMALL_FAST_MODEL)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(get_default_haiku_model)
}

/// Check if model is a non-custom Opus model
pub fn is_non_custom_opus_model(model: &ModelName) -> bool {
    model == &get_model_strings().opus_40
        || model == &get_model_strings().opus_41
        || model == &get_model_strings().opus_45
        || model == &get_model_strings().opus_46
}

/// Get user-specified model from environment or settings
/// Returns the model name or alias if specified, None if not configured
pub fn get_user_specified_model_setting() -> Option<String> {
    // First check for model override (would come from bootstrap/state)
    if let Some(override_model) = get_main_loop_model_override() {
        if is_model_allowed(&override_model) {
            return Some(override_model);
        } else {
            return None;
        }
    }

    // Check environment variable
    if let Ok(env_model) = std::env::var(ai::MODEL) {
        if !env_model.is_empty() && is_model_allowed(&env_model) {
            return Some(env_model);
        }
    }

    // Check settings (stub - would need settings module)
    // let settings = get_settings_deprecated() or {};
    // if settings.model && is_model_allowed(settings.model) { return Some(settings.model) }

    None
}

/// Get the main loop model to use for the current session.
/// Model Selection Priority Order:
/// 1. Model override during session (from /model command) - highest priority
/// 2. Model override at startup (from --model flag)
/// 3. ANTHROPIC_MODEL environment variable
/// 4. Settings (from user's saved settings)
/// 5. Built-in default
pub fn get_main_loop_model() -> ModelName {
    if let Some(model) = get_user_specified_model_setting() {
        return parse_user_specified_model(model);
    }
    get_default_main_loop_model()
}

/// Get best model (Opus)
pub fn get_best_model() -> ModelName {
    get_default_opus_model()
}

/// Get default Opus model
pub fn get_default_opus_model() -> ModelName {
    std::env::var(ai::DEFAULT_OPUS_MODEL)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| get_model_strings().opus_46.clone())
}

/// Get default Sonnet model
pub fn get_default_sonnet_model() -> ModelName {
    if let Ok(model) = std::env::var(ai::DEFAULT_SONNET_MODEL) {
        if !model.is_empty() {
            return model;
        }
    }

    // For 3P providers, use older Sonnet
    if get_api_provider() != "firstParty" {
        return get_model_strings().sonnet_45.clone();
    }
    get_model_strings().sonnet_46.clone()
}

/// Get default Haiku model
pub fn get_default_haiku_model() -> ModelName {
    std::env::var(ai::DEFAULT_HAIKU_MODEL)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| get_model_strings().haiku_45.clone())
}

/// Get runtime main loop model depending on runtime context
pub fn get_runtime_main_loop_model(
    permission_mode: &str,
    main_loop_model: &str,
    exceeds_200k_tokens: bool,
) -> ModelName {
    // opusplan uses Opus in plan mode without [1m] suffix
    if get_user_specified_model_setting() == Some("opusplan".to_string())
        && permission_mode == "plan"
        && !exceeds_200k_tokens
    {
        return get_default_opus_model();
    }

    // sonnetplan by default
    if get_user_specified_model_setting() == Some("haiku".to_string()) && permission_mode == "plan"
    {
        return get_default_sonnet_model();
    }

    main_loop_model.to_string()
}

/// Get default main loop model setting
pub fn get_default_main_loop_model_setting() -> ModelNameOrAlias {
    // Ants default to defaultModel from flag config, or Opus 1M if not configured
    if let Ok(user_type) = std::env::var(ai::USER_TYPE) {
        if user_type == "ant" {
            if let Some(ant_config) = get_ant_model_override_config() {
                return ant_config.default_model;
            }
            return format!("{}[1m]", get_default_opus_model());
        }
    }

    // Max users get Opus as default
    if is_max_subscriber() {
        return if is_opus_1m_merge_enabled() {
            format!("{}[1m]", get_default_opus_model())
        } else {
            get_default_opus_model()
        };
    }

    // Team Premium gets Opus (same as Max)
    if is_team_premium_subscriber() {
        return if is_opus_1m_merge_enabled() {
            format!("{}[1m]", get_default_opus_model())
        } else {
            get_default_opus_model()
        };
    }

    // PAYG, Enterprise, Team Standard, and Pro get Sonnet as default
    get_default_sonnet_model()
}

/// Get default main loop model (synchronous)
pub fn get_default_main_loop_model() -> ModelName {
    parse_user_specified_model(get_default_main_loop_model_setting())
}

/// Convert first-party model name to canonical short name
pub fn first_party_name_to_canonical(name: &ModelName) -> ModelShortName {
    let name_lower = name.to_lowercase();

    // Special cases for Claude 4+ models
    if name_lower.contains("claude-opus-4-6") {
        return "claude-opus-4-6".to_string();
    }
    if name_lower.contains("claude-opus-4-5") {
        return "claude-opus-4-5".to_string();
    }
    if name_lower.contains("claude-opus-4-1") {
        return "claude-opus-4-1".to_string();
    }
    if name_lower.contains("claude-opus-4") {
        return "claude-opus-4".to_string();
    }
    if name_lower.contains("claude-sonnet-4-6") {
        return "claude-sonnet-4-6".to_string();
    }
    if name_lower.contains("claude-sonnet-4-5") {
        return "claude-sonnet-4-5".to_string();
    }
    if name_lower.contains("claude-sonnet-4") {
        return "claude-sonnet-4".to_string();
    }
    if name_lower.contains("claude-haiku-4-5") {
        return "claude-haiku-4-5".to_string();
    }

    // Claude 3.x models
    if name_lower.contains("claude-3-7-sonnet") {
        return "claude-3-7-sonnet".to_string();
    }
    if name_lower.contains("claude-3-5-sonnet") {
        return "claude-3-5-sonnet".to_string();
    }
    if name_lower.contains("claude-3-5-haiku") {
        return "claude-3-5-haiku".to_string();
    }
    if name_lower.contains("claude-3-opus") {
        return "claude-3-opus".to_string();
    }
    if name_lower.contains("claude-3-sonnet") {
        return "claude-3-sonnet".to_string();
    }
    if name_lower.contains("claude-3-haiku") {
        return "claude-3-haiku".to_string();
    }

    // Fallback pattern match
    if let Some(captures) = regex::Regex::new(r"(claude-(\d+-\d+-)?\w+)")
        .ok()
        .and_then(|re| re.captures(&name_lower))
    {
        if let Some(m) = captures.get(1) {
            return m.as_str().to_string();
        }
    }

    // Fall back to original name
    name.clone()
}

/// Get canonical name from full model string
pub fn get_canonical_name(full_model_name: &str) -> ModelShortName {
    let resolved = resolve_overridden_model(full_model_name);
    first_party_name_to_canonical(&resolved)
}

/// Get Claude AI user default model description
pub fn get_claude_ai_user_default_model_description(fast_mode: bool) -> String {
    if is_max_subscriber() || is_team_premium_subscriber() {
        let base = if is_opus_1m_merge_enabled() {
            "Opus 4.6 with 1M context"
        } else {
            "Opus 4.6"
        };
        let suffix = if fast_mode {
            get_opus_46_pricing_suffix(true)
        } else {
            "".to_string()
        };
        format!("{} · Most capable for complex work{}", base, suffix)
    } else {
        "Sonnet 4.6 · Best for everyday tasks".to_string()
    }
}

/// Render default model setting for display
pub fn render_default_model_setting(setting: &ModelNameOrAlias) -> String {
    if setting == "opusplan" {
        return "Opus 4.6 in plan mode, else Sonnet 4.6".to_string();
    }
    render_model_name(&parse_user_specified_model(setting.clone()))
}

/// Get Opus 4.6 pricing suffix
pub fn get_opus_46_pricing_suffix(fast_mode: bool) -> String {
    if get_api_provider() != "firstParty" {
        return "".to_string();
    }
    // Would need model_cost module for actual pricing
    let pricing = "pricing_placeholder".to_string();
    let fast_mode_indicator = if fast_mode { " (lightning)" } else { "" };
    format!(" ·{} {}", fast_mode_indicator, pricing)
}

/// Check if Opus 1M merge is enabled
pub fn is_opus_1m_merge_enabled() -> bool {
    if is_1m_context_disabled() || is_pro_subscriber() || get_api_provider() != "firstParty" {
        return false;
    }

    // Fail closed when subscription type is unknown
    if is_claude_ai_subscriber() && get_subscription_type().is_none() {
        return false;
    }

    true
}

/// Render model setting for display
pub fn render_model_setting(setting: &ModelNameOrAlias) -> String {
    if setting == "opusplan" {
        return "Opus Plan".to_string();
    }
    if is_model_alias(setting) {
        return capitalize(setting);
    }
    render_model_name(setting)
}

/// Get public model display name
pub fn get_public_model_display_name(model: &ModelName) -> Option<String> {
    let model_strings = get_model_strings();

    if model == &model_strings.opus_46 {
        return Some("Opus 4.6".to_string());
    }
    if model == &format!("{}[1m]", model_strings.opus_46) {
        return Some("Opus 4.6 (1M context)".to_string());
    }
    if model == &model_strings.opus_45 {
        return Some("Opus 4.5".to_string());
    }
    if model == &model_strings.opus_41 {
        return Some("Opus 4.1".to_string());
    }
    if model == &model_strings.opus_40 {
        return Some("Opus 4".to_string());
    }
    if model == &format!("{}[1m]", model_strings.sonnet_46) {
        return Some("Sonnet 4.6 (1M context)".to_string());
    }
    if model == &model_strings.sonnet_46 {
        return Some("Sonnet 4.6".to_string());
    }
    if model == &format!("{}[1m]", model_strings.sonnet_45) {
        return Some("Sonnet 4.5 (1M context)".to_string());
    }
    if model == &model_strings.sonnet_45 {
        return Some("Sonnet 4.5".to_string());
    }
    if model == &model_strings.sonnet_40 {
        return Some("Sonnet 4".to_string());
    }
    if model == &format!("{}[1m]", model_strings.sonnet_40) {
        return Some("Sonnet 4 (1M context)".to_string());
    }
    if model == &model_strings.sonnet_37 {
        return Some("Sonnet 3.7".to_string());
    }
    if model == &model_strings.sonnet_35 {
        return Some("Sonnet 3.5".to_string());
    }
    if model == &model_strings.haiku_45 {
        return Some("Haiku 4.5".to_string());
    }
    if model == &model_strings.haiku_35 {
        return Some("Haiku 3.5".to_string());
    }

    None
}

/// Mask model codename for display
fn mask_model_codename(base_name: &str) -> String {
    let parts: Vec<&str> = base_name.split('-').collect();
    if parts.is_empty() {
        return base_name.to_string();
    }

    let codename = parts[0];
    let rest: Vec<&str> = parts[1..].to_vec();

    let masked = if codename.len() > 3 {
        format!("{}{}", &codename[..3], "*".repeat(codename.len() - 3))
    } else {
        codename.to_string()
    };

    let mut result = masked;
    for part in rest {
        result.push('-');
        result.push_str(part);
    }
    result
}

/// Render model name for display
pub fn render_model_name(model: &ModelName) -> String {
    if let Some(public_name) = get_public_model_display_name(model) {
        return public_name;
    }

    if let Ok(user_type) = std::env::var(ai::USER_TYPE) {
        if user_type == "ant" {
            let resolved = parse_user_specified_model(model.clone());
            if let Some(ant_model) = resolve_ant_model(model) {
                let base_name = ant_model.model.replace("[1m]", "");
                let masked = mask_model_codename(&base_name);
                let suffix = if has_1m_context(&resolved) {
                    "[1m]"
                } else {
                    ""
                };
                return format!("{}{}", masked, suffix);
            }
            if resolved != *model {
                return format!("{} ({})", model, resolved);
            }
            return resolved;
        }
    }

    model.clone()
}

/// Get public model name for display (e.g., in git commit trailers)
pub fn get_public_model_name(model: &ModelName) -> String {
    if let Some(public_name) = get_public_model_display_name(model) {
        return format!("Claude {}", public_name);
    }
    format!("Claude ({})", model)
}

/// Parse user specified model and return full model name
pub fn parse_user_specified_model(model_input: ModelNameOrAlias) -> ModelName {
    let model_input_trimmed = model_input.trim().to_string();
    let normalized_model = model_input_trimmed.to_lowercase();

    let has_1m_tag = has_1m_context(&normalized_model);
    let model_string = if has_1m_tag {
        normalized_model.replace("[1m]", "").trim().to_string()
    } else {
        normalized_model.clone()
    };

    if is_model_alias(&model_string) {
        match model_string.as_str() {
            "opusplan" => {
                return format!(
                    "{}{}",
                    get_default_sonnet_model(),
                    if has_1m_tag { "[1m]" } else { "" }
                );
            }
            "sonnet" => {
                return format!(
                    "{}{}",
                    get_default_sonnet_model(),
                    if has_1m_tag { "[1m]" } else { "" }
                );
            }
            "haiku" => {
                return format!(
                    "{}{}",
                    get_default_haiku_model(),
                    if has_1m_tag { "[1m]" } else { "" }
                );
            }
            "opus" => {
                return format!(
                    "{}{}",
                    get_default_opus_model(),
                    if has_1m_tag { "[1m]" } else { "" }
                );
            }
            "best" => {
                return get_best_model();
            }
            _ => {}
        }
    }

    // Legacy Opus remap for first-party API
    if get_api_provider() == "firstParty"
        && is_legacy_opus_first_party(&model_string)
        && is_legacy_model_remap_enabled()
    {
        return format!(
            "{}{}",
            get_default_opus_model(),
            if has_1m_tag { "[1m]" } else { "" }
        );
    }

    // Handle ant models
    if let Ok(user_type) = std::env::var(ai::USER_TYPE) {
        if user_type == "ant" {
            let has_1m_ant_tag = has_1m_context(&normalized_model);
            let base_ant_model = normalized_model.replace("[1m]", "").trim().to_string();

            if let Some(ant_model) = resolve_ant_model(&base_ant_model) {
                let suffix = if has_1m_ant_tag { "[1m]" } else { "" };
                return format!("{}{}", ant_model.model, suffix);
            }
        }
    }

    // Preserve original case for custom model names
    if has_1m_tag {
        return format!("{}[1m]", model_input_trimmed.replace("[1m]", "").trim());
    }
    model_input_trimmed
}

/// Resolve skill model override
pub fn resolve_skill_model_override(skill_model: &str, current_model: &str) -> String {
    if has_1m_context(skill_model) || !has_1m_context(current_model) {
        return skill_model.to_string();
    }

    if model_supports_1m(&parse_user_specified_model(skill_model.to_string())) {
        return format!("{}[1m]", skill_model);
    }
    skill_model.to_string()
}

/// Legacy Opus first-party models
const LEGACY_OPUS_FIRSTPARTY: &[&str] = &[
    "claude-opus-4-20250514",
    "claude-opus-4-1-20250805",
    "claude-opus-4-0",
    "claude-opus-4-1",
];

fn is_legacy_opus_first_party(model: &str) -> bool {
    LEGACY_OPUS_FIRSTPARTY.contains(&model)
}

/// Check if legacy model remap is enabled
pub fn is_legacy_model_remap_enabled() -> bool {
    !is_env_truthy(&std::env::var(ai_code::DISABLE_LEGACY_MODEL_REMAP).unwrap_or_default())
}

/// Model display string
pub fn model_display_string(model: &ModelSetting) -> String {
    if model.is_none() {
        if let Ok(user_type) = std::env::var(ai::USER_TYPE) {
            if user_type == "ant" {
                return format!(
                    "Default for Ants ({})",
                    render_default_model_setting(&get_default_main_loop_model_setting())
                );
            }
        }
        if is_claude_ai_subscriber() {
            return format!(
                "Default ({})",
                get_claude_ai_user_default_model_description(false)
            );
        }
        return format!("Default ({})", get_default_main_loop_model());
    }

    let model = model.as_ref().unwrap();
    let resolved_model = parse_user_specified_model(model.clone());
    if model == &resolved_model {
        resolved_model
    } else {
        format!("{} ({})", model, resolved_model)
    }
}

/// Get marketing name for model
pub fn get_marketing_name_for_model(model_id: &str) -> Option<String> {
    if get_api_provider() == "foundry" {
        return None;
    }

    let has_1m = model_id.to_lowercase().contains("[1m]");
    let canonical = get_canonical_name(model_id);

    if canonical.contains("claude-opus-4-6") {
        return Some(if has_1m {
            "Opus 4.6 (with 1M context)".to_string()
        } else {
            "Opus 4.6".to_string()
        });
    }
    if canonical.contains("claude-opus-4-5") {
        return Some("Opus 4.5".to_string());
    }
    if canonical.contains("claude-opus-4-1") {
        return Some("Opus 4.1".to_string());
    }
    if canonical.contains("claude-opus-4") {
        return Some("Opus 4".to_string());
    }
    if canonical.contains("claude-sonnet-4-6") {
        return Some(if has_1m {
            "Sonnet 4.6 (with 1M context)".to_string()
        } else {
            "Sonnet 4.6".to_string()
        });
    }
    if canonical.contains("claude-sonnet-4-5") {
        return Some(if has_1m {
            "Sonnet 4.5 (with 1M context)".to_string()
        } else {
            "Sonnet 4.5".to_string()
        });
    }
    if canonical.contains("claude-sonnet-4") {
        return Some(if has_1m {
            "Sonnet 4 (with 1M context)".to_string()
        } else {
            "Sonnet 4".to_string()
        });
    }
    if canonical.contains("claude-3-7-sonnet") {
        return Some("Claude 3.7 Sonnet".to_string());
    }
    if canonical.contains("claude-3-5-sonnet") {
        return Some("Claude 3.5 Sonnet".to_string());
    }
    if canonical.contains("claude-haiku-4-5") {
        return Some("Haiku 4.5".to_string());
    }
    if canonical.contains("claude-3-5-haiku") {
        return Some("Claude 3.5 Haiku".to_string());
    }

    None
}

/// Normalize model string for API (removes [1m] or [2m] suffix)
pub fn normalize_model_string_for_api(model: &str) -> String {
    regex::Regex::new(r"\[(1|2)m\]")
        .map(|re| re.replace_all(model, "").to_string())
        .unwrap_or_else(|_| model.to_string())
}

// =============================================================================
// STUB HELPERS - These need to be implemented with actual module dependencies
// =============================================================================

/// Model strings cache
static MODEL_STRINGS: OnceLock<ModelStrings> = OnceLock::new();

#[derive(Debug, Clone)]
struct ModelStrings {
    opus_40: ModelName,
    opus_41: ModelName,
    opus_45: ModelName,
    opus_46: ModelName,
    sonnet_35: ModelName,
    sonnet_37: ModelName,
    sonnet_40: ModelName,
    sonnet_45: ModelName,
    sonnet_46: ModelName,
    haiku_35: ModelName,
    haiku_45: ModelName,
}

fn get_model_strings() -> &'static ModelStrings {
    MODEL_STRINGS.get_or_init(|| ModelStrings {
        opus_40: "claude-opus-4-0-20250514".to_string(),
        opus_41: "claude-opus-4-1-20250805".to_string(),
        opus_45: "claude-opus-4-5-20250514".to_string(),
        opus_46: "claude-opus-4-6-20251106".to_string(),
        sonnet_35: "claude-sonnet-3-5-20241022".to_string(),
        sonnet_37: "claude-sonnet-3-7-20250120".to_string(),
        sonnet_40: "claude-sonnet-4-0-20250514".to_string(),
        sonnet_45: "claude-sonnet-4-5-20241022".to_string(),
        sonnet_46: "claude-sonnet-4-6-20251106".to_string(),
        haiku_35: "claude-haiku-3-5-20241022".to_string(),
        haiku_45: "claude-haiku-4-5-20250513".to_string(),
    })
}

/// Get API provider
fn get_api_provider() -> String {
    std::env::var(ai::API_PROVIDER)
        .ok()
        .unwrap_or_else(|| "firstParty".to_string())
}

/// Get main loop model override (from bootstrap/state)
fn get_main_loop_model_override() -> Option<ModelName> {
    // Stub - would need to check bootstrap state
    None
}

/// Check if model is allowed based on availableModels allowlist in settings.
/// Returns true if no restrictions are configured (conservative default).
/// Note: Full implementation would read from settings files (managed by enterprise admins).
fn is_model_allowed(_model: &str) -> bool {
    // TODO: Full implementation would read availableModels from settings.json
    // For now, allow all models (conservative - restrict only if explicitly blocked)
    true
}

/// Check if model string is an alias
fn is_model_alias(model: &str) -> bool {
    matches!(model, "opus" | "sonnet" | "haiku" | "opusplan" | "best")
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Check if 1M context is disabled
fn is_1m_context_disabled() -> bool {
    // Stub - would need context module
    false
}

/// Check if model has 1M context tag
fn has_1m_context(model: &str) -> bool {
    model.to_lowercase().ends_with("[1m]")
}

/// Check if model supports 1M context
fn model_supports_1m(model: &ModelName) -> bool {
    // Stub - would need context module
    let canonical = get_canonical_name(model);
    matches!(
        canonical.as_str(),
        "claude-opus-4-6" | "claude-opus-4-5" | "claude-sonnet-4-6" | "claude-sonnet-4-5"
    )
}

/// Resolve overridden model (e.g., Bedrock ARNs)
fn resolve_overridden_model(model: &str) -> ModelName {
    // Stub - would need modelStrings module
    model.to_string()
}

/// Check if user is max subscriber
fn is_max_subscriber() -> bool {
    get_subscription_type() == Some("max".to_string())
}

/// Check if user is team premium subscriber
fn is_team_premium_subscriber() -> bool {
    get_subscription_type() == Some("team".to_string())
        && get_rate_limit_tier() == Some("default_claude_max_5x".to_string())
}

/// Check if user is pro subscriber
fn is_pro_subscriber() -> bool {
    get_subscription_type() == Some("pro".to_string())
}

/// Get rate limit tier from OAuth tokens
fn get_rate_limit_tier() -> Option<String> {
    use crate::session_history::get_claude_ai_oauth_tokens;
    get_claude_ai_oauth_tokens().and_then(|t| t.rate_limit_tier.clone())
}

/// Check if user is Claude AI subscriber (Max/Pro with OAuth)
/// Returns true if user has OAuth tokens with user scope (not just user:inference).
pub fn is_claude_ai_subscriber() -> bool {
    use crate::session_history::get_claude_ai_oauth_tokens;
    use crate::utils::env_utils::is_env_truthy;

    // Check if 3rd-party auth is enabled (never subscriber in that case)
    if is_env_truthy(Some("AI_CODE_USE_BEDROCK"))
        || is_env_truthy(Some("AI_CODE_USE_VERTEX"))
        || is_env_truthy(Some("AI_CODE_USE_FOUNDRY"))
    {
        return false;
    }

    // Check OAuth token presence with user scope
    if let Some(tokens) = get_claude_ai_oauth_tokens() {
        return tokens.scopes.iter().any(|s| s.contains("user")) && !tokens.access_token.is_empty();
    }

    false
}

/// Get subscription type (max, pro, team, or None for API/PAYG)
pub fn get_subscription_type() -> Option<String> {
    use crate::session_history::get_claude_ai_oauth_tokens;

    get_claude_ai_oauth_tokens().and_then(|t| t.subscription_type.clone())
}

/// Check environment variable is truthy
fn is_env_truthy(value: &str) -> bool {
    let normalized = value.to_lowercase();
    matches!(normalized.trim(), "1" | "true" | "yes" | "on")
}

/// Ant model config
#[derive(Debug, Clone)]
struct AntModelConfig {
    default_model: String,
    model: String,
}

/// Get ant model override config
fn get_ant_model_override_config() -> Option<AntModelConfig> {
    // Stub - would need antModels module
    None
}

/// Resolve ant model
fn resolve_ant_model(_model: &str) -> Option<AntModelConfig> {
    // Stub - would need antModels module
    None
}
