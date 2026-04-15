//! Session environment utilities.

use crate::constants::env::ai;
use std::collections::HashMap;

/// Get environment variables for a session
pub fn get_session_environment() -> HashMap<String, String> {
    let mut env = HashMap::new();

    // Add session-specific environment variables
    if let Ok(session_id) = std::env::var(ai::CODE_SESSION_ID) {
        env.insert(ai::CODE_SESSION_ID.to_string(), session_id);
    }

    if let Ok(model) = std::env::var(ai::MODEL) {
        env.insert(ai::MODEL.to_string(), model);
    }

    // Add all environment variables starting with AI_CODE_
    for (key, value) in std::env::vars() {
        if key.starts_with("AI_CODE_") {
            env.insert(key, value);
        }
    }

    env
}

/// Set session environment variables
pub fn set_session_environment(env: &HashMap<String, String>) {
    for (key, value) in env {
        unsafe { std::env::set_var(key, value) };
    }
}

/// Clear session environment variables
pub fn clear_session_environment() {
    let keys: Vec<String> = std::env::vars()
        .filter(|(key, _)| key.starts_with("AI_CODE_"))
        .map(|(key, _)| key)
        .collect();

    for key in keys {
        unsafe { std::env::remove_var(&key) };
    }
}
