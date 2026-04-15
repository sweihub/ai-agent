// Source: /data/home/swei/claudecode/openclaudecode/src/utils/env.ts
//! ENV configuration reader
//!
//! Reads configuration from .env file and environment variables.
//! Supports AI_* prefixed variables for SDK configuration.

use crate::constants::env::ai;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration values from environment
#[derive(Debug, Clone, Default)]
pub struct EnvConfig {
    /// Base URL for the AI API
    pub base_url: Option<String>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Model to use
    pub model: Option<String>,
    /// Additional raw env values (AI_* prefixed)
    pub extras: HashMap<String, String>,
}

impl EnvConfig {
    /// Load env config from .env file and environment variables
    /// Searches for .env in: current directory, then parent directories, then exe directory
    /// Also loads from ~/.ai/settings.json
    pub fn load() -> Self {
        // First try current directory
        let mut config = Self::load_from_dir(".");

        // If no config found, try the executable's directory
        if config.base_url.is_none() && config.auth_token.is_none() && config.model.is_none() {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let exe_config = Self::load_from_dir(exe_dir.to_str().unwrap_or("."));
                    // Merge if we found something
                    if exe_config.base_url.is_some()
                        || exe_config.auth_token.is_some()
                        || exe_config.model.is_some()
                    {
                        config = exe_config;
                    }
                }
            }
        }

        // Also try loading from ~/.ai/settings.json
        if let Some(home_dir) = dirs::home_dir() {
            let settings_path = home_dir.join(".ai").join("settings.json");
            if settings_path.exists() {
                if let Ok(content) = fs::read_to_string(&settings_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(env) = json.get("env").and_then(|v| v.as_object()) {
                            for (key, value) in env {
                                if let Some(v) = value.as_str() {
                                    config.set_value(key, v);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Override with system environment variables
        config.load_from_env();

        config
    }

    /// Load env config from a specific directory
    pub fn load_from_dir(dir: &str) -> Self {
        let mut config = Self::default();

        // Try to load from .env file
        let env_path = Path::new(dir).join(".env");
        if env_path.exists() {
            if let Ok(content) = fs::read_to_string(&env_path) {
                config.parse_env_file(&content);
            }
        }

        // Also check parent directories (up to 3 levels)
        let mut current = Path::new(dir);
        for _ in 0..3 {
            if let Some(parent) = current.parent() {
                let parent_env = parent.join(".env");
                if parent_env.exists() && parent_env != env_path {
                    if let Ok(content) = fs::read_to_string(&parent_env) {
                        config.parse_env_file(&content);
                    }
                }
                current = parent;
            } else {
                break;
            }
        }

        // Override with system environment variables
        config.load_from_env();

        config
    }

    /// Parse .env file content
    fn parse_env_file(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Remove quotes if present
                let value = value.trim_matches('"').trim_matches('\'');

                self.set_value(key, value);
            }
        }
    }

    /// Load from system environment variables
    fn load_from_env(&mut self) {
        // AI_BASE_URL (SDK native)
        if let Ok(val) = std::env::var(ai::BASE_URL) {
            self.base_url = Some(val);
        }

        // AI_AUTH_TOKEN (SDK native)
        if let Ok(val) = std::env::var(ai::AUTH_TOKEN) {
            self.auth_token = Some(val);
        }

        // ANTHROPIC_API_KEY fallback - check if no AI_AUTH_TOKEN
        if self.auth_token.is_none() {
            if let Ok(val) = std::env::var(ai::API_KEY) {
                self.auth_token = Some(val);
            }
        }

        // ANTHROPIC_AUTH_TOKEN fallback - check if no auth_token yet
        if self.auth_token.is_none() {
            if let Ok(val) = std::env::var(ai::ANTHROPIC_AUTH_TOKEN) {
                self.auth_token = Some(val);
            }
        }

        // AI_MODEL (SDK native)
        if let Ok(val) = std::env::var(ai::MODEL) {
            self.model = Some(val);
        }

        // ANTHROPIC_MODEL fallback - check if no AI_MODEL
        if self.model.is_none() {
            if let Ok(val) = std::env::var(ai::ANTHROPIC_MODEL) {
                self.model = Some(val);
            }
        }

        // Any other AI_* variables
        for (key, value) in std::env::vars() {
            if key.starts_with("AI_") {
                match key.as_str() {
                    "AI_BASE_URL" | "AI_AUTH_TOKEN" | "AI_MODEL" => {} // Already handled
                    _ => {
                        self.extras.insert(key, value);
                    }
                }
            }
        }
    }

    /// Set a configuration value
    fn set_value(&mut self, key: &str, value: &str) {
        match key {
            "AI_BASE_URL" => self.base_url = Some(value.to_string()),
            ai::BASE_URL => {
                if self.base_url.is_none() {
                    self.base_url = Some(value.to_string());
                }
            }
            "AI_AUTH_TOKEN" => self.auth_token = Some(value.to_string()),
            ai::API_KEY | ai::AUTH_TOKEN => {
                // Only set if no auth_token already (AI_AUTH_TOKEN takes priority)
                if self.auth_token.is_none() {
                    self.auth_token = Some(value.to_string());
                }
            }
            "AI_MODEL" => self.model = Some(value.to_string()),
            ai::MODEL => {
                if self.model.is_none() {
                    self.model = Some(value.to_string());
                }
            }
            _ => {
                if key.starts_with("AI_") {
                    self.extras.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&str> {
        match key {
            "AI_BASE_URL" => self.base_url.as_deref(),
            "AI_AUTH_TOKEN" => self.auth_token.as_deref(),
            "AI_MODEL" => self.model.as_deref(),
            _ => self.extras.get(key).map(|s| s.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_file() {
        let mut config = EnvConfig::default();
        config.parse_env_file(
            r#"
# Comment
AI_BASE_URL="http://localhost:8000"
AI_AUTH_TOKEN='test-token'
AI_MODEL=claude-sonnet-4-6
"#,
        );

        assert_eq!(config.base_url, Some("http://localhost:8000".to_string()));
        assert_eq!(config.auth_token, Some("test-token".to_string()));
        assert_eq!(config.model, Some("claude-sonnet-4-6".to_string()));
    }

    #[test]
    fn test_get_values() {
        let config = EnvConfig {
            base_url: Some("http://test".to_string()),
            auth_token: Some("token".to_string()),
            model: Some("model".to_string()),
            extras: HashMap::new(),
        };

        assert_eq!(config.get("AI_BASE_URL"), Some("http://test"));
        assert_eq!(config.get("AI_AUTH_TOKEN"), Some("token"));
        assert_eq!(config.get("AI_MODEL"), Some("model"));
        assert_eq!(config.get("UNKNOWN"), None);
    }
}

// =============================================================================
// ASSISTANT MODE
// =============================================================================

/// Read the assistant mode flag from environment variables.
/// Checks AI_CODE_ASSISTANT_MODE.
fn read_assistant_mode_flag() -> bool {
    if let Ok(val) = std::env::var(ai::CODE_ASSISTANT_MODE) {
        return val == "1" || val == "true";
    }

    false
}

/// Check if assistant mode is enabled.
/// This function is used to determine if the CLI is running in assistant mode.
pub fn is_assistant_mode() -> bool {
    read_assistant_mode_flag()
}

/// Check if assistant mode is enabled (alias for is_assistant_mode).
pub fn is_assistant_mode_enabled() -> bool {
    read_assistant_mode_flag()
}
