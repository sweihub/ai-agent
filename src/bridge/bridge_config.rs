//! Shared bridge auth/URL resolution.
//!
//! Translated from openclaudecode/src/bridge/bridgeApi.ts
//!
//! Consolidates the ant-only CLAUDE_BRIDGE_* dev overrides.

use std::env;
use std::sync::OnceLock;

// =============================================================================
// STATIC GETTERS
// =============================================================================

static AUTH_TOKEN_GETTER: OnceLock<Box<dyn Fn() -> Option<String> + Send + Sync>> = OnceLock::new();
static BASE_URL_GETTER: OnceLock<Box<dyn Fn() -> String + Send + Sync>> = OnceLock::new();

/// Register the OAuth token getter function.
pub fn register_oauth_token_getter(getter: impl Fn() -> Option<String> + Send + Sync + 'static) {
    let _ = AUTH_TOKEN_GETTER.set(Box::new(getter));
}

/// Register the base URL getter function.
pub fn register_oauth_base_url_getter(getter: impl Fn() -> String + Send + Sync + 'static) {
    let _ = BASE_URL_GETTER.set(Box::new(getter));
}

/// Get the OAuth config base URL.
fn get_oauth_config_base_url() -> String {
    BASE_URL_GETTER
        .get()
        .map(|getter| getter())
        .unwrap_or_else(|| "https://api.anthropic.com".to_string())
}

/// Get the OAuth access token.
fn get_oauth_access_token() -> Option<String> {
    AUTH_TOKEN_GETTER.get().and_then(|getter| getter())
}

// =============================================================================
// ANT-ONLY DEV OVERRIDES
// =============================================================================

/// Ant-only dev override: CLAUDE_BRIDGE_OAUTH_TOKEN, else None.
pub fn get_bridge_token_override() -> Option<String> {
    if is_ant_user() {
        env::var("CLAUDE_BRIDGE_OAUTH_TOKEN").ok()
    } else {
        None
    }
}

/// Ant-only dev override: CLAUDE_BRIDGE_BASE_URL, else None.
pub fn get_bridge_base_url_override() -> Option<String> {
    if is_ant_user() {
        env::var("CLAUDE_BRIDGE_BASE_URL").ok()
    } else {
        None
    }
}

/// Check if running as ant user.
fn is_ant_user() -> bool {
    env::var("USER_TYPE").map(|v| v == "ant").unwrap_or(false)
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Access token for bridge API calls: dev override first, then the OAuth
/// keychain. None means "not logged in".
pub fn get_bridge_access_token() -> Option<String> {
    get_bridge_token_override().or_else(get_oauth_access_token)
}

/// Base URL for bridge API calls: dev override first, then the production
/// OAuth config. Always returns a URL.
pub fn get_bridge_base_url() -> String {
    get_bridge_base_url_override().unwrap_or_else(get_oauth_config_base_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_access_token_no_override() {
        // Without env vars set, should return None or OAuth token
        let token = get_bridge_access_token();
        // Just ensure it doesn't panic
        let _ = token;
    }

    #[test]
    fn test_bridge_base_url_default() {
        let url = get_bridge_base_url();
        assert!(!url.is_empty());
    }
}
