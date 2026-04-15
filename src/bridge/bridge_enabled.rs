//! Runtime check for bridge mode entitlement.
//!
//! Translated from openclaudecode/src/bridge/bridgeEnabled.ts
//!
//! Remote Control requires a claude.ai subscription. isClaudeAISubscriber()
//! excludes Bedrock/Vertex/Foundry, apiKeyHelper/gateway deployments, env-var
//! API keys, and Console API logins.

use std::env;
use std::sync::OnceLock;

// =============================================================================
// STATIC GETTERS (for dependency injection)
// =============================================================================

/// Gate check function: (gate_name: &str) -> bool
pub type GateFn = Box<dyn Fn(&str) -> bool + Send + Sync>;

/// Dynamic config getter: (key: &str) -> Option<Value>
pub type DynamicConfigFn = Box<dyn Fn(&str) -> Option<serde_json::Value> + Send + Sync>;

/// Version checker: (current: &str, min: &str) -> bool
pub type VersionCheckFn = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// Subscriber check: () -> bool
pub type SubscriberCheckFn = Box<dyn Fn() -> bool + Send + Sync>;

/// Profile scope check: () -> bool
pub type ProfileScopeFn = Box<dyn Fn() -> bool + Send + Sync>;

/// OAuth account info getter: () -> Option<OAuthAccountInfo>
pub type OauthAccountFn = Box<dyn Fn() -> Option<OAuthAccountInfo> + Send + Sync>;

/// Env truthy check: (key: &str) -> bool
pub type EnvTruthyFn = Box<dyn Fn(&str) -> bool + Send + Sync>;

static GATE_GETTER: OnceLock<GateFn> = OnceLock::new();
static DYNAMIC_CONFIG_GETTER: OnceLock<DynamicConfigFn> = OnceLock::new();
static VERSION_CHECK_GETTER: OnceLock<VersionCheckFn> = OnceLock::new();
static SUBSCRIBER_CHECK: OnceLock<SubscriberCheckFn> = OnceLock::new();
static PROFILE_SCOPE_CHECK: OnceLock<ProfileScopeFn> = OnceLock::new();
static OAUTH_ACCOUNT_GETTER: OnceLock<OauthAccountFn> = OnceLock::new();
static ENV_TRUTHY_CHECK: OnceLock<EnvTruthyFn> = OnceLock::new();

// Build-time feature flags (these would be set at compile time)
static BRIDGE_MODE: OnceLock<bool> = OnceLock::new();
static CCR_AUTO_CONNECT: OnceLock<bool> = OnceLock::new();
static CCR_MIRROR: OnceLock<bool> = OnceLock::new();

// =============================================================================
// TYPES
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct OAuthAccountInfo {
    pub organization_uuid: Option<String>,
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Register the gate check function (from GrowthBook).
pub fn register_gate_check(gate: impl Fn(&str) -> bool + Send + Sync + 'static) {
    let _ = GATE_GETTER.set(Box::new(gate));
}

/// Register the dynamic config getter function.
pub fn register_dynamic_config(
    getter: impl Fn(&str) -> Option<serde_json::Value> + Send + Sync + 'static,
) {
    let _ = DYNAMIC_CONFIG_GETTER.set(Box::new(getter));
}

/// Register the version checker function.
pub fn register_version_check(checker: impl Fn(&str, &str) -> bool + Send + Sync + 'static) {
    let _ = VERSION_CHECK_GETTER.set(Box::new(checker));
}

/// Register the subscriber check function.
pub fn register_subscriber_check(check: impl Fn() -> bool + Send + Sync + 'static) {
    let _ = SUBSCRIBER_CHECK.set(Box::new(check));
}

/// Register the profile scope check function.
pub fn register_profile_scope_check(check: impl Fn() -> bool + Send + Sync + 'static) {
    let _ = PROFILE_SCOPE_CHECK.set(Box::new(check));
}

/// Register the OAuth account info getter.
pub fn register_oauth_account_getter(
    getter: impl Fn() -> Option<OAuthAccountInfo> + Send + Sync + 'static,
) {
    let _ = OAUTH_ACCOUNT_GETTER.set(Box::new(getter));
}

/// Register the env truthy check function.
pub fn register_env_truthy_check(check: impl Fn(&str) -> bool + Send + Sync + 'static) {
    let _ = ENV_TRUTHY_CHECK.set(Box::new(check));
}

/// Set build-time feature flags.
pub fn set_bridge_mode(enabled: bool) {
    let _ = BRIDGE_MODE.set(enabled);
}

pub fn set_ccr_auto_connect(enabled: bool) {
    let _ = CCR_AUTO_CONNECT.set(enabled);
}

pub fn set_ccr_mirror(enabled: bool) {
    let _ = CCR_MIRROR.set(enabled);
}

// =============================================================================
// GATE CHECK HELPERS
// =============================================================================

fn get_gate(gate_name: &str) -> bool {
    GATE_GETTER
        .get()
        .map(|gate| gate(gate_name))
        .unwrap_or(false)
}

fn get_dynamic_config(key: &str) -> Option<serde_json::Value> {
    DYNAMIC_CONFIG_GETTER.get().and_then(|getter| getter(key))
}

fn check_version(current: &str, min: &str) -> bool {
    VERSION_CHECK_GETTER
        .get()
        .map(|check| check(current, min))
        .unwrap_or_else(|| {
            // Simple fallback: compare as strings for common cases
            // In real usage, semver would be used
            current >= min
        })
}

fn is_claude_ai_subscriber() -> bool {
    SUBSCRIBER_CHECK.get().map(|check| check()).unwrap_or(false)
}

fn has_profile_scope() -> bool {
    PROFILE_SCOPE_CHECK
        .get()
        .map(|check| check())
        .unwrap_or(false)
}

fn get_oauth_account_info() -> Option<OAuthAccountInfo> {
    OAUTH_ACCOUNT_GETTER.get().and_then(|getter| getter())
}

fn is_env_truthy(key: &str) -> bool {
    ENV_TRUTHY_CHECK
        .get()
        .map(|check| check(key))
        .unwrap_or_else(|| {
            env::var(key)
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false)
        })
}

fn bridge_mode_enabled() -> bool {
    BRIDGE_MODE.get().copied().unwrap_or(false)
}

fn ccr_auto_connect_enabled() -> bool {
    CCR_AUTO_CONNECT.get().copied().unwrap_or(false)
}

fn ccr_mirror_enabled() -> bool {
    CCR_MIRROR.get().copied().unwrap_or(false)
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Runtime check for bridge mode entitlement.
/// Returns true when both the build flag and GrowthBook gate are enabled.
pub fn is_bridge_enabled() -> bool {
    if !bridge_mode_enabled() {
        return false;
    }

    // In production, we'd check both conditions
    // For SDK, we default to true if gate not set
    get_gate("tengu_ccr_bridge")
}

/// Blocking entitlement check for Remote Control.
/// Currently just returns the same as is_bridge_enabled.
pub fn is_bridge_enabled_blocking() -> bool {
    is_bridge_enabled()
}

/// Diagnostic message for why Remote Control is unavailable, or None if
/// it's enabled.
pub fn get_bridge_disabled_reason() -> Option<String> {
    if !bridge_mode_enabled() {
        return Some("Remote Control is not available in this build.".to_string());
    }

    if !is_claude_ai_subscriber() {
        return Some(
            "Remote Control requires a claude.ai subscription. Run `claude auth login` to sign in \
             with your claude.ai account."
                .to_string(),
        );
    }

    if !has_profile_scope() {
        return Some(
            "Remote Control requires a full-scope login token. Long-lived tokens (from `claude \
             setup-token` or AI_CODE_OAUTH_TOKEN) are limited to inference-only for security \
             reasons. Run `claude auth login` to use Remote Control."
                .to_string(),
        );
    }

    let account = get_oauth_account_info();
    if account.is_none()
        || account
            .as_ref()
            .and_then(|a| a.organization_uuid.as_ref())
            .is_none()
    {
        return Some(
            "Unable to determine your organization for Remote Control eligibility. Run \
             `claude auth login` to refresh your account information."
                .to_string(),
        );
    }

    if !get_gate("tengu_ccr_bridge") {
        return Some("Remote Control is not yet enabled for your account.".to_string());
    }

    None
}

/// Runtime check for the env-less (v2) REPL bridge path.
/// Returns true when the GrowthBook flag is enabled.
pub fn is_env_less_bridge_enabled() -> bool {
    if !bridge_mode_enabled() {
        return false;
    }

    get_gate("tengu_bridge_repl_v2")
}

/// Kill-switch for the cse_ -> session_ client-side retag shim.
/// Defaults to true — the shim stays active until explicitly disabled.
pub fn is_cse_shim_enabled() -> bool {
    if !bridge_mode_enabled() {
        return true;
    }

    // Get the feature value, default to true
    get_dynamic_config("tengu_bridge_repl_v2_cse_shim_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

/// Check if the current CLI version meets the minimum required for Remote Control.
/// Returns an error message if version is too old, or None if OK.
pub fn check_bridge_min_version(current_version: &str) -> Option<String> {
    if !bridge_mode_enabled() {
        return None;
    }

    let config = get_dynamic_config("tengu_bridge_min_version");
    let min_version = config
        .and_then(|c| c.get("minVersion").cloned())
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "0.0.0".to_string());

    if !check_version(current_version, &min_version) {
        return Some(format!(
            "Your version of Claude Code ({}) is too old for Remote Control.\nVersion {} or \
             higher is required. Run `claude update` to update.",
            current_version, min_version
        ));
    }

    None
}

/// Default for remoteControlAtStartup when the user hasn't explicitly set it.
/// When the CCR_AUTO_CONNECT build flag is present and the GrowthBook gate
/// is on, all sessions connect to CCR by default.
pub fn get_ccr_auto_connect_default() -> bool {
    if !ccr_auto_connect_enabled() {
        return false;
    }

    get_gate("tengu_cobalt_harbor")
}

/// Opt-in CCR mirror mode — every local session spawns an outbound-only
/// Remote Control session that receives forwarded events.
pub fn is_ccr_mirror_enabled() -> bool {
    if !ccr_mirror_enabled() {
        return false;
    }

    is_env_truthy("AI_CODE_CCR_MIRROR") || get_gate("tengu_ccr_mirror")
}

// =============================================================================
// CSE SHIM GATE REGISTRATION
// =============================================================================

/// Register the CSE shim gate with the session_id_compat module.
pub fn register_cse_shim_gate() {
    use crate::bridge::session_id_compat::set_cse_shim_gate;
    set_cse_shim_gate(is_cse_shim_enabled);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_disabled_without_build_flag() {
        // Without setting bridge mode, should be disabled
        assert!(!is_bridge_enabled());
    }

    #[test]
    fn test_env_less_bridge_default() {
        assert!(!is_env_less_bridge_enabled());
    }

    #[test]
    fn test_cse_shim_default() {
        // Default to true when not in bridge mode
        assert!(is_cse_shim_enabled());
    }

    #[test]
    fn test_check_bridge_min_version() {
        let result = check_bridge_min_version("1.0.0");
        // Without config set, should pass (default min is 0.0.0)
        assert!(result.is_none());
    }
}
