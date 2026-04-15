//! Session start utilities.

use serde::{Deserialize, Serialize};

/// Session start configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartConfig {
    pub model: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl Default for SessionStartConfig {
    fn default() -> Self {
        Self {
            model: None,
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        }
    }
}

/// Create a new session
pub fn create_session(config: SessionStartConfig) -> SessionStartResult {
    // Generate a new session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    SessionStartResult { session_id, config }
}

/// Result of session creation
#[derive(Debug, Clone)]
pub struct SessionStartResult {
    pub session_id: String,
    pub config: SessionStartConfig,
}
