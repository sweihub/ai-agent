// Source: /data/home/swei/claudecode/openclaudecode/src/components/CustomSelect/select.tsx
use crate::constants::env::{ai, anthropic};
use crate::env::EnvConfig;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub type ModelName = String;
pub type ModelShortName = String;
pub type ModelAlias = String;
pub type ModelSetting = Option<String>;

#[derive(Debug, Clone, Default)]
pub struct ModelStrings {
    pub opus46: String,
    pub opus45: String,
    pub opus41: String,
    pub opus40: String,
    pub sonnet46: String,
    pub sonnet45: String,
    pub sonnet40: String,
    pub sonnet37: String,
    pub sonnet35: String,
    pub haiku45: String,
    pub haiku35: String,
}

impl ModelStrings {
    pub fn default_first_party() -> Self {
        Self {
            opus46: "claude-opus-4-6-20250514".to_string(),
            opus45: "claude-opus-4-5-20250514".to_string(),
            opus41: "claude-opus-4-1-20250514".to_string(),
            opus40: "claude-opus-4-0-20250514".to_string(),
            sonnet46: "claude-sonnet-4-6-20250514".to_string(),
            sonnet45: "claude-sonnet-4-5-20250514".to_string(),
            sonnet40: "claude-sonnet-4-0-20250514".to_string(),
            sonnet37: "claude-sonnet-3-7-20250514".to_string(),
            sonnet35: "claude-sonnet-3-5-20250603".to_string(),
            haiku45: "claude-haiku-4-5-20250514".to_string(),
            haiku35: "claude-haiku-3-5-20250514".to_string(),
        }
    }

    pub fn get() -> Self {
        MODEL_STRINGS.lock().unwrap().clone()
    }
}

static MODEL_STRINGS: Lazy<Mutex<ModelStrings>> = Lazy::new(|| {
    let env_config = EnvConfig::load();
    Mutex::new(ModelStrings::default_first_party())
});

pub const MODEL_ALIASES: &[&str] = &["opus", "sonnet", "haiku", "opusplan", "best"];

pub fn is_model_alias(model: &str) -> bool {
    MODEL_ALIASES.contains(&model.to_lowercase().as_str())
}

pub fn get_small_fast_model() -> ModelName {
    std::env::var(ai::SMALL_FAST_MODEL)
        .ok()
        .unwrap_or_else(get_default_haiku_model)
}

pub fn is_non_custom_opus_model(model: &ModelName) -> bool {
    let strings = ModelStrings::get();
    model == strings.opus40
        || model == strings.opus41
        || model == strings.opus45
        || model == strings.opus46
}

pub fn get_user_specified_model_setting() -> ModelSetting {
    let env_config = EnvConfig::load();

    if let Some(model) = &env_config.model {
        if is_model_allowed(model) {
            return Some(model.clone());
        }
        return None;
    }

    None
}

pub fn is_model_allowed(model: &str) -> bool {
    let lower = model.to_lowercase();

    if is_model_alias(&lower) {
        return true;
    }

    let strings = ModelStrings::get();
    let known_models = [
        strings.opus46.as_str(),
        strings.opus45.as_str(),
        strings.opus41.as_str(),
        strings.opus40.as_str(),
        strings.sonnet46.as_str(),
        strings.sonnet45.as_str(),
        strings.sonnet40.as_str(),
        strings.sonnet37.as_str(),
        strings.sonnet35.as_str(),
        strings.haiku45.as_str(),
        strings.haiku35.as_str(),
    ];

    if known_models.contains(&lower.as_str()) {
        return true;
    }
    if known_models.contains(&lower.replace("[1m]", "").as_str()) {
        return true;
    }

    let prefixes = ["claude-opus-", "claude-sonnet-", "claude-haiku-"];
    for prefix in prefixes {
        if lower.starts_with(prefix) {
            return true;
        }
    }

    false
}

pub fn get_main_loop_model() -> ModelName {
    if let Some(model) = get_user_specified_model_setting() {
        return parse_user_specified_model(&model);
    }
    get_default_main_loop_model()
}

pub fn get_best_model() -> ModelName {
    get_default_opus_model()
}

pub fn get_default_opus_model() -> ModelName {
    if let Ok(model) = std::env::var(anthropic::DEFAULT_OPUS_MODEL) {
        return model;
    }
    ModelStrings::get().opus46
}

pub fn get_default_sonnet_model() -> ModelName {
    if let Ok(model) = std::env::var(anthropic::DEFAULT_SONNET_MODEL) {
        return model;
    }
    ModelStrings::get().sonnet46
}

pub fn get_default_haiku_model() -> ModelName {
    if let Ok(model) = std::env::var(anthropic::DEFAULT_HAIKU_MODEL) {
        return model;
    }
    ModelStrings::get().haiku45
}

pub fn get_default_main_loop_model() -> ModelName {
    parse_user_specified_model(&get_default_main_loop_model_setting())
}

pub fn get_default_main_loop_model_setting() -> String {
    get_default_sonnet_model()
}

pub fn parse_user_specified_model(model_input: &str) -> ModelName {
    let model_input = model_input.trim();
    let normalized = model_input.to_lowercase();

    let has_1m = normalized.contains("[1m]");
    let model_base = if has_1m {
        normalized.replace("[1m]", "").trim().to_string()
    } else {
        normalized.clone()
    };

    if is_model_alias(&model_base) {
        match model_base.as_str() {
            "opusplan" => {
                let base = get_default_sonnet_model();
                return if has_1m {
                    format!("{}[1m]", base)
                } else {
                    base
                };
            }
            "sonnet" => {
                let base = get_default_sonnet_model();
                return if has_1m {
                    format!("{}[1m]", base)
                } else {
                    base
                };
            }
            "haiku" => {
                let base = get_default_haiku_model();
                return if has_1m {
                    format!("{}[1m]", base)
                } else {
                    base
                };
            }
            "opus" => {
                let base = get_default_opus_model();
                return if has_1m {
                    format!("{}[1m]", base)
                } else {
                    base
                };
            }
            "best" => {
                return get_best_model();
            }
            _ => {}
        }
    }

    let legacy_opus = [
        "claude-opus-4-20250514",
        "claude-opus-4-1-20250805",
        "claude-opus-4-0",
        "claude-opus-4-1",
    ];
    if legacy_opus.contains(&model_base.as_str()) {
        let base = get_default_opus_model();
        return if has_1m {
            format!("{}[1m]", base)
        } else {
            base
        };
    }

    if has_1m {
        let base = model_input.replace("[1m]", "").trim();
        return format!("{}[1m]", base);
    }
    model_input.to_string()
}

pub fn first_party_name_to_canonical(name: &str) -> ModelShortName {
    let name_lower = name.to_lowercase();

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

    if let Some(cap) = regex::Regex::new(r"(claude-(\d+-\d+-)?\w+)")
        .ok()
        .and_then(|re| re.captures(&name_lower))
    {
        if let Some(m) = cap.get(1) {
            return m.as_str().to_string();
        }
    }

    name.to_string()
}

pub fn get_canonical_name(full_model_name: &str) -> ModelShortName {
    first_party_name_to_canonical(full_model_name)
}

pub fn get_public_model_display_name(model: &str) -> Option<String> {
    let strings = ModelStrings::get();

    match model {
        s if s == strings.opus46 => Some("Opus 4.6".to_string()),
        s if s == format!("{}[1m]", strings.opus46) => Some("Opus 4.6 (1M context)".to_string()),
        s if s == strings.opus45 => Some("Opus 4.5".to_string()),
        s if s == strings.opus41 => Some("Opus 4.1".to_string()),
        s if s == strings.opus40 => Some("Opus 4".to_string()),
        s if s == format!("{}[1m]", strings.sonnet46) => {
            Some("Sonnet 4.6 (1M context)".to_string())
        }
        s if s == strings.sonnet46 => Some("Sonnet 4.6".to_string()),
        s if s == format!("{1}[1m]", strings.sonnet45) => {
            Some("Sonnet 4.5 (1M context)".to_string())
        }
        s if s == strings.sonnet45 => Some("Sonnet 4.5".to_string()),
        s if s == strings.sonnet40 => Some("Sonnet 4".to_string()),
        s if s == format!("{}[1m]", strings.sonnet40) => Some("Sonnet 4 (1M context)".to_string()),
        s if s == strings.sonnet37 => Some("Sonnet 3.7".to_string()),
        s if s == strings.sonnet35 => Some("Sonnet 3.5".to_string()),
        s if s == strings.haiku45 => Some("Haiku 4.5".to_string()),
        s if s == strings.haiku35 => Some("Haiku 3.5".to_string()),
        _ => None,
    }
}

pub fn render_model_name(model: &ModelName) -> String {
    if let Some(public_name) = get_public_model_display_name(model) {
        return public_name;
    }
    model.clone()
}

pub fn normalize_model_string_for_api(model: &str) -> String {
    let re = regex::Regex::new(r"\[(1|2)m\]").unwrap();
    re.replace_all(model, "").to_string()
}

pub fn model_supports_1m(model: &str) -> bool {
    let strings = ModelStrings::get();
    let canonical = get_canonical_name(model);

    canonical.contains("claude-opus-4") || canonical.contains("claude-sonnet-4")
}

pub fn has_1m_context(model: &str) -> bool {
    model.to_lowercase().contains("[1m]")
}

pub fn resolve_skill_model_override(skill_model: &str, current_model: &str) -> String {
    if has_1m_context(skill_model) || !has_1m_context(current_model) {
        return skill_model.to_string();
    }

    let parsed = parse_user_specified_model(skill_model);
    if model_supports_1m(&parsed) {
        return format!("{}[1m]", skill_model);
    }

    skill_model.to_string()
}
