//! Session ingress authentication utilities.

use crate::constants::env::ai;
use serde::{Deserialize, Serialize};

/// Ingress authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressAuthConfig {
    pub enabled: bool,
    pub token: Option<String>,
    pub oauth_token: Option<String>,
}

/// Check if ingress authentication is required
pub fn is_ingress_auth_required() -> bool {
    std::env::var(ai::CODE_INGRESS_AUTH_REQUIRED)
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Get ingress authentication token
pub fn get_ingress_token() -> Option<String> {
    std::env::var(ai::CODE_INGRESS_TOKEN).ok()
}

/// Validate an ingress token
pub fn validate_ingress_token(token: &str) -> bool {
    // Compare with configured token
    if let Some(configured) = get_ingress_token() {
        return token == configured;
    }

    // If no token configured, accept any non-empty token
    !token.is_empty()
}
