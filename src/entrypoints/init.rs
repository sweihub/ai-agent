// Source: /data/home/swei/claudecode/openclaudecode/src/entrypoints/init.ts
use crate::constants::env::ai;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfig {
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            model: None,
            api_key: None,
            base_url: None,
            max_tokens: Some(4096),
            temperature: Some(1.0),
        }
    }
}

impl InitConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }
}

pub fn initialize(config: InitConfig) -> Result<(), String> {
    if config.api_key.is_none() && std::env::var(ai::API_KEY).is_err() {
        return Err("API key is required".to_string());
    }
    Ok(())
}
