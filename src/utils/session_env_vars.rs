//! Session environment variables utilities.

use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;

/// Get environment variables from the session configuration
pub fn get_session_environment() -> HashMap<String, String> {
    let mut env = HashMap::new();

    // Add session-specific environment variables
    if let Ok(session_id) = env::var("AI_CODE_SESSION_ID") {
        env.insert("AI_CODE_SESSION_ID".to_string(), session_id);
    }

    if let Ok(model) = env::var("AI_CODE_MODEL") {
        env.insert("AI_CODE_MODEL".to_string(), model);
    }

    // Add all environment variables starting with AI_CODE_
    for (key, value) in env::vars() {
        if key.starts_with("AI_CODE_") {
            env.insert(key, value);
        }
    }

    env
}

/// Set environment variables for the session
pub fn set_session_environment(env_vars: &HashMap<String, String>) {
    for (key, value) in env_vars {
        unsafe { env::set_var(key, value) };
    }
}

/// Clear session environment variables
pub fn clear_session_environment() {
    let keys: Vec<String> = env::vars()
        .filter(|(key, _)| key.starts_with("AI_CODE_"))
        .map(|(key, _)| key)
        .collect();

    for key in keys {
        unsafe { env::remove_var(&key) };
    }
}

/// Parse environment variable string (key=value pairs) into a map
pub fn parse_env_vars(env_str: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    for line in env_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            vars.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    vars
}
