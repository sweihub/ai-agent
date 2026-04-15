use super::select::{get_public_model_display_name, ModelStrings};
use crate::services::model_cost::{ModelCosts, COST_TIER_15_75, COST_TIER_3_15, COST_TIER_5_25};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    Opus,
    Sonnet,
    Haiku,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAvailability {
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub context_window: u32,
    pub max_output_tokens: Option<u32>,
    pub training_cutoff: Option<String>,
    pub supports_vision: bool,
    pub supports_1m: bool,
    pub tier: ModelTier,
    pub cost_tier: String,
}

impl ModelMetadata {
    pub fn get_costs(&self) -> &'static ModelCosts {
        match self.tier {
            ModelTier::Opus => &COST_TIER_15_75,
            ModelTier::Sonnet => &COST_TIER_5_25,
            ModelTier::Haiku => &COST_TIER_3_15,
        }
    }
}

pub fn get_available_models() -> Vec<ModelMetadata> {
    let strings = ModelStrings::get();

    vec![
        ModelMetadata {
            id: strings.opus46.clone(),
            display_name: "Opus 4.6".to_string(),
            description: "Most capable model for complex tasks".to_string(),
            context_window: 1_000_000,
            max_output_tokens: Some(64_000),
            training_cutoff: Some("2025-05".to_string()),
            supports_vision: true,
            supports_1m: true,
            tier: ModelTier::Opus,
            cost_tier: "15/75".to_string(),
        },
        ModelMetadata {
            id: strings.opus45.clone(),
            display_name: "Opus 4.5".to_string(),
            description: "Highly capable for complex work".to_string(),
            context_window: 200_000,
            max_output_tokens: Some(64_000),
            training_cutoff: Some("2025-02".to_string()),
            supports_vision: true,
            supports_1m: false,
            tier: ModelTier::Opus,
            cost_tier: "15/75".to_string(),
        },
        ModelMetadata {
            id: strings.sonnet46.clone(),
            display_name: "Sonnet 4.6".to_string(),
            description: "Best balance of capability and speed".to_string(),
            context_window: 1_000_000,
            max_output_tokens: Some(64_000),
            training_cutoff: Some("2025-05".to_string()),
            supports_vision: true,
            supports_1m: true,
            tier: ModelTier::Sonnet,
            cost_tier: "3/15".to_string(),
        },
        ModelMetadata {
            id: strings.sonnet45.clone(),
            display_name: "Sonnet 4.5".to_string(),
            description: "Great for everyday tasks".to_string(),
            context_window: 200_000,
            max_output_tokens: Some(64_000),
            training_cutoff: Some("2025-02".to_string()),
            supports_vision: true,
            supports_1m: false,
            tier: ModelTier::Sonnet,
            cost_tier: "3/15".to_string(),
        },
        ModelMetadata {
            id: strings.haiku45.clone(),
            display_name: "Haiku 4.5".to_string(),
            description: "Fast and efficient for simple tasks".to_string(),
            context_window: 200_000,
            max_output_tokens: Some(64_000),
            training_cutoff: Some("2025-05".to_string()),
            supports_vision: true,
            supports_1m: false,
            tier: ModelTier::Haiku,
            cost_tier: "0.25/1.25".to_string(),
        },
    ]
}

pub fn get_models_by_tier(tier: ModelTier) -> Vec<ModelMetadata> {
    get_available_models()
        .into_iter()
        .filter(|m| m.tier == tier)
        .collect()
}

pub fn get_opus_models() -> Vec<ModelMetadata> {
    get_models_by_tier(ModelTier::Opus)
}

pub fn get_sonnet_models() -> Vec<ModelMetadata> {
    get_models_by_tier(ModelTier::Sonnet)
}

pub fn get_haiku_models() -> Vec<ModelMetadata> {
    get_models_by_tier(ModelTier::Haiku)
}

pub fn is_model_available(model_id: &str) -> ModelAvailability {
    let available_models = get_available_models();

    for model in available_models {
        if model.id == model_id {
            return ModelAvailability {
                available: true,
                reason: None,
            };
        }
    }

    if model_id.starts_with("claude-") {
        return ModelAvailability {
            available: true,
            reason: None,
        };
    }

    ModelAvailability {
        available: false,
        reason: Some("Model not recognized".to_string()),
    }
}

pub fn get_model_by_id(model_id: &str) -> Option<ModelMetadata> {
    let available_models = get_available_models();

    for model in available_models {
        if model.id == model_id {
            return Some(model);
        }
    }

    let strings = ModelStrings::get();
    if model_id.contains("opus-4-6") {
        return Some(ModelMetadata {
            id: strings.opus46,
            display_name: "Opus 4.6".to_string(),
            description: "Most capable model".to_string(),
            context_window: 1_000_000,
            max_output_tokens: Some(64_000),
            training_cutoff: Some("2025-05".to_string()),
            supports_vision: true,
            supports_1m: true,
            tier: ModelTier::Opus,
            cost_tier: "15/75".to_string(),
        });
    }

    if model_id.contains("sonnet-4-6") {
        return Some(ModelMetadata {
            id: strings.sonnet46,
            display_name: "Sonnet 4.6".to_string(),
            description: "Best balance".to_string(),
            context_window: 1_000_000,
            max_output_tokens: Some(64_000),
            training_cutoff: Some("2025-05".to_string()),
            supports_vision: true,
            supports_1m: true,
            tier: ModelTier::Sonnet,
            cost_tier: "3/15".to_string(),
        });
    }

    if model_id.contains("haiku") {
        return Some(ModelMetadata {
            id: strings.haiku45,
            display_name: "Haiku".to_string(),
            description: "Fast model".to_string(),
            context_window: 200_000,
            max_output_tokens: Some(64_000),
            training_cutoff: Some("2025-05".to_string()),
            supports_vision: true,
            supports_1m: false,
            tier: ModelTier::Haiku,
            cost_tier: "0.25/1.25".to_string(),
        });
    }

    None
}

pub fn format_model_for_display(model_id: &str) -> String {
    if let Some(metadata) = get_model_by_id(model_id) {
        return metadata.display_name;
    }

    get_public_model_display_name(model_id).unwrap_or_else(|| model_id.to_string())
}

pub fn get_models_by_context_size(min_context: u32) -> Vec<ModelMetadata> {
    get_available_models()
        .into_iter()
        .filter(|m| m.context_window >= min_context)
        .collect()
}

pub fn get_vision_models() -> Vec<ModelMetadata> {
    get_available_models()
        .into_iter()
        .filter(|m| m.supports_vision)
        .collect()
}

pub fn get_1m_models() -> Vec<ModelMetadata> {
    get_available_models()
        .into_iter()
        .filter(|m| m.supports_1m)
        .collect()
}
