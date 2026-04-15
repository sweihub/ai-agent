//! Model 1M access checks.
//!
//! Translated from openclaudecode/src/utils/model/check1mAccess.ts

use crate::utils::config::get_global_config;
use crate::utils::env_utils::is_env_truthy;

// =============================================================================
// EXTRA USAGE CHECK
// =============================================================================

/// Check if extra usage is enabled based on the cached disabled reason.
/// Extra usage is considered enabled if there's no disabled reason,
/// or if the disabled reason indicates it's provisioned but temporarily unavailable.
fn is_extra_usage_enabled() -> bool {
    let config = get_global_config();
    let reason = config.cached_extra_usage_disabled_reason.clone();

    // None = no cache yet, treat as not enabled (conservative)
    let reason = match reason {
        Some(r) => r,
        None => return false,
    };

    // Empty string = no disabled reason from API, extra usage is enabled
    if reason.is_empty() {
        return true;
    }

    // Check which disabled reasons still mean "provisioned"
    // "out_of_credits" = provisioned but credits depleted — still counts as enabled
    if reason == "out_of_credits" {
        return true;
    }

    // All other reasons mean not provisioned or actively disabled
    false
}

// =============================================================================
// 1M ACCESS CHECKS
// =============================================================================

/// Check if Opus 1M context access is available
pub fn check_opus_1m_access() -> bool {
    if is_1m_context_disabled() {
        return false;
    }

    if is_claude_ai_subscriber() {
        // Subscribers have access if extra usage is enabled for their account
        return is_extra_usage_enabled();
    }

    // Non-subscribers (API/PAYG) have access
    true
}

/// Check if Sonnet 1M context access is available
pub fn check_sonnet_1m_access() -> bool {
    if is_1m_context_disabled() {
        return false;
    }

    if is_claude_ai_subscriber() {
        // Subscribers have access if extra usage is enabled for their account
        return is_extra_usage_enabled();
    }

    // Non-subscribers (API/PAYG) have access
    true
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Check if 1M context is disabled
/// Returns true if user has disabled 1M context window (GrowthBook override)
fn is_1m_context_disabled() -> bool {
    // Check GrowthBook feature gate if available
    // For SDK, this is controlled via AI_CODE_DISABLE_1M_CONTEXT env var
    is_env_truthy(Some("AI_CODE_DISABLE_1M_CONTEXT"))
}

/// Check if user is Claude AI subscriber (Max/Pro with OAuth)
pub fn is_claude_ai_subscriber() -> bool {
    use crate::session_history::get_claude_ai_oauth_tokens;

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