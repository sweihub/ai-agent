// Source: /data/home/swei/claudecode/openclaudecode/src/utils/billing.ts
//! Billing utilities
//!
//! Translated from openclaudecode/src/utils/billing.ts

use crate::constants::env::{ai, ai_code};
use crate::utils::config::GlobalConfig;
use crate::utils::env_utils::is_env_truthy;

/// Check if the user has console billing access
pub fn has_console_billing_access() -> bool {
    // Check if cost reporting is disabled via environment variable
    if is_env_truthy(Some("DISABLE_COST_WARNINGS")) {
        return false;
    }

    let is_subscriber = is_claude_ai_subscriber();

    // This might be wrong if user is signed into Max but also using an API key, but
    // we already show a warning on launch in that case
    if is_subscriber {
        return false;
    }

    // Check if user has any form of authentication
    let (_auth_source, has_token) = get_auth_token_source();
    let has_api_key = get_anthropic_api_key().is_some();

    // If user has no authentication at all (logged out), don't show costs
    if !has_token && !has_api_key {
        return false;
    }

    let config = GlobalConfig::default();
    let org_role = config
        .oauth_account
        .as_ref()
        .and_then(|a| a.organization_role.as_ref());
    let workspace_role = config
        .oauth_account
        .as_ref()
        .and_then(|a| a.workspace_role.as_ref());

    if org_role.is_none() || workspace_role.is_none() {
        // Hide cost for grandfathered users who have not re-authed since we've added roles
        return false;
    }

    let org_role = org_role.unwrap();
    let workspace_role = workspace_role.unwrap();

    // Users have billing access if they are admins or billing roles at either workspace or organization level
    ["admin", "billing"].contains(&org_role.as_str())
        || ["workspace_admin", "workspace_billing"].contains(&workspace_role.as_str())
}

// Mock billing access for /mock-limits testing (set by mockRateLimits.ts)
static MOCK_BILLING_ACCESS_OVERRIDE: std::sync::Mutex<Option<bool>> = std::sync::Mutex::new(None);

/// Set mock billing access override for testing
pub fn set_mock_billing_access_override(value: Option<bool>) {
    let mut mock = MOCK_BILLING_ACCESS_OVERRIDE.lock().unwrap();
    *mock = value;
}

/// Check if the user has Claude AI billing access
pub fn has_claude_ai_billing_access() -> bool {
    // Check for mock billing access first (for /mock-limits testing)
    let mock = MOCK_BILLING_ACCESS_OVERRIDE.lock().unwrap();
    if let Some(override_value) = *mock {
        return override_value;
    }
    drop(mock);

    if !is_claude_ai_subscriber() {
        return false;
    }

    let subscription_type = get_subscription_type();

    // Consumer plans (Max/Pro) - individual users always have billing access
    if subscription_type == Some("max".to_string()) || subscription_type == Some("pro".to_string())
    {
        return true;
    }

    // Team/Enterprise - check for admin or billing roles
    let config = GlobalConfig::default();
    let org_role = config
        .oauth_account
        .as_ref()
        .and_then(|a| a.organization_role.as_ref());

    org_role
        .map(|role| ["admin", "billing", "owner", "primary_owner"].contains(&role.as_str()))
        .unwrap_or(false)
}

// Re-implement the functions from auth module to avoid import issues
fn is_claude_ai_subscriber() -> bool {
    // Check for subscription status via environment or config
    if let Ok(token) = std::env::var(ai::AUTH_TOKEN) {
        if !token.is_empty() {
            return true;
        }
    }
    if let Ok(token) = std::env::var(ai_code::OAUTH_TOKEN) {
        if !token.is_empty() {
            return true;
        }
    }
    false
}

fn get_auth_token_source() -> (String, bool) {
    if std::env::var(ai::AUTH_TOKEN).is_ok() {
        return (ai::AUTH_TOKEN.to_string(), true);
    }
    if std::env::var(ai_code::OAUTH_TOKEN).is_ok() {
        return (ai_code::OAUTH_TOKEN.to_string(), true);
    }
    ("none".to_string(), false)
}

fn get_anthropic_api_key() -> Option<String> {
    std::env::var(ai::API_KEY).ok().filter(|k| !k.is_empty())
}

fn get_subscription_type() -> Option<String> {
    // This would need more implementation based on the actual auth system
    None
}
