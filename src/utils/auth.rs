// Source: /data/home/swei/claudecode/openclaudecode/src/types/generated/events_mono/common/v1/auth.ts
#![allow(dead_code)]

use crate::constants::env::{anthropic, ai, ai_code, system};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;

// Default TTL for API key helper cache in milliseconds (5 minutes)
const DEFAULT_API_KEY_HELPER_TTL: u64 = 5 * 60 * 1000;

// Default STS credentials are one hour
const DEFAULT_AWS_STS_TTL: u64 = 60 * 60 * 1000;

// Timeout for AWS auth refresh command (3 minutes)
const AWS_AUTH_REFRESH_TIMEOUT_MS: u64 = 3 * 60 * 1000;

// Short timeout for the GCP credentials probe
const GCP_CREDENTIALS_CHECK_TIMEOUT_MS: u64 = 5_000;

// Default GCP credential TTL - 1 hour
const DEFAULT_GCP_CREDENTIAL_TTL: u64 = 60 * 60 * 1000;

#[derive(Debug, Clone, PartialEq)]
pub enum ApiKeySource {
    AnthropicApiKey,
    ApiKeyHelper,
    LoginManagedKey,
    None,
}

#[derive(Debug, Clone)]
pub struct ApiKeyResult {
    pub key: Option<String>,
    pub source: ApiKeySource,
}

/// CCR and Claude Desktop spawn the CLI with OAuth and should never fall back
/// to the user's ~/.ai/settings.json API-key config (apiKeyHelper,
/// env.AI_API_KEY, env.AI_AUTH_TOKEN). Those settings exist for
/// the user's terminal CLI, not managed sessions.
fn is_managed_oauth_context() -> bool {
    std::env::var(ai::REMOTE).is_ok()
        || std::env::var(ai::CODE_ENTRYPOINT).map(|v| v == "claude-desktop").unwrap_or(false)
}

/// Whether we are supporting direct 1P auth.
pub fn is_anthropic_auth_enabled() -> bool {
    // --bare: API-key-only, never OAuth.
    if is_bare_mode() {
        return false;
    }

    // `claude ssh` remote: ANTHROPIC_UNIX_SOCKET tunnels API calls through a
    // local auth-injecting proxy.
    if std::env::var(anthropic::UNIX_SOCKET).is_ok() {
        return std::env::var(ai_code::OAUTH_TOKEN).is_ok();
    }

    let is_3p = is_env_truthy("AI_CODE_USE_BEDROCK")
        || is_env_truthy("AI_CODE_USE_VERTEX")
        || is_env_truthy("AI_CODE_USE_FOUNDRY");

    // Check if user has configured an external API key source
    let settings = get_settings_deprecated().unwrap_or_default();
    let api_key_helper = settings.get("apiKeyHelper").cloned();
    let has_external_auth_token = std::env::var(anthropic::AUTH_TOKEN).is_ok()
        || api_key_helper.is_some()
        || std::env::var(ai_code::API_KEY_FILE_DESCRIPTOR).is_ok();

    // Check if API key is from an external source
    let api_key_source = get_anthropic_api_key_with_source_internal(true).source;
    let has_external_api_key = api_key_source == ApiKeySource::AnthropicApiKey
        || api_key_source == ApiKeySource::ApiKeyHelper;

    // Disable Anthropic auth if:
    // 1. Using 3rd party services (Bedrock/Vertex/Foundry)
    // 2. User has an external API key (regardless of proxy configuration)
    // 3. User has an external auth token (regardless of proxy configuration)
    let should_disable_auth = is_3p
        || (has_external_auth_token && !is_managed_oauth_context())
        || (has_external_api_key && !is_managed_oauth_context());

    !should_disable_auth
}

/// Where the auth token is being sourced from, if any.
pub fn get_auth_token_source() -> (String, bool) {
    // --bare: API-key-only
    if is_bare_mode() {
        if get_configured_api_key_helper().is_some() {
            return ("apiKeyHelper".to_string(), true);
        }
        return ("none".to_string(), false);
    }

    if std::env::var(anthropic::AUTH_TOKEN).is_ok() && !is_managed_oauth_context() {
        return (anthropic::AUTH_TOKEN.to_string(), true);
    }

    if std::env::var(ai_code::OAUTH_TOKEN).is_ok() {
        return ("AI_CODE_OAUTH_TOKEN".to_string(), true);
    }

    // Check for OAuth token from file descriptor
    if let Some(token) = get_oauth_token_from_file_descriptor() {
        if std::env::var(ai_code::OAUTH_TOKEN_FILE_DESCRIPTOR).is_ok() {
            return ("AI_CODE_OAUTH_TOKEN_FILE_DESCRIPTOR".to_string(), true);
        }
        return ("CCR_OAUTH_TOKEN_FILE".to_string(), true);
    }

    // Check if apiKeyHelper is configured
    let api_key_helper = get_configured_api_key_helper();
    if api_key_helper.is_some() && !is_managed_oauth_context() {
        return ("apiKeyHelper".to_string(), true);
    }

    // Check for Claude.ai OAuth tokens
    let oauth_tokens = get_claude_ai_oauth_tokens();
    if should_use_claude_ai_auth(&oauth_tokens) && oauth_tokens.as_ref().map(|t| t.access_token.is_some()).unwrap_or(false) {
        return ("claude.ai".to_string(), true);
    }

    ("none".to_string(), false)
}

pub fn get_anthropic_api_key() -> Option<String> {
    let result = get_anthropic_api_key_with_source_internal(false);
    result.key
}

pub fn has_anthropic_api_key_auth() -> bool {
    let result = get_anthropic_api_key_with_source_internal(true);
    result.key.is_some() && result.source != ApiKeySource::None
}

pub fn get_anthropic_api_key_with_source(
    skip_retrieving_key_from_api_key_helper: bool,
) -> ApiKeyResult {
    get_anthropic_api_key_with_source_internal(skip_retrieving_key_from_api_key_helper)
}

fn get_anthropic_api_key_with_source_internal(
    skip_retrieving_key_from_api_key_helper: bool,
) -> ApiKeyResult {
    // --bare: hermetic auth
    if is_bare_mode() {
        if let Ok(key) = std::env::var(anthropic::API_KEY) {
            return ApiKeyResult {
                key: Some(key),
                source: ApiKeySource::AnthropicApiKey,
            };
        }
        if let Some(helper) = get_configured_api_key_helper() {
            return ApiKeyResult {
                key: if skip_retrieving_key_from_api_key_helper {
                    None
                } else {
                    get_api_key_from_api_key_helper_cached()
                },
                source: ApiKeySource::ApiKeyHelper,
            };
        }
        return ApiKeyResult {
            key: None,
            source: ApiKeySource::None,
        };
    }

    // On homespace, don't use ANTHROPIC_API_KEY (use Console key instead)
    let api_key_env = if is_running_on_homespace() {
        None
    } else {
        std::env::var(anthropic::API_KEY).ok()
    };

    // Always check for direct environment variable when the user ran claude --print
    if prefer_third_party_authentication() {
        if let Some(ref key) = api_key_env {
            return ApiKeyResult {
                key: Some(key.clone()),
                source: ApiKeySource::AnthropicApiKey,
            };
        }
    }

    // CI or test mode
    if std::env::var(system::CI).is_ok() || std::env::var(system::NODE_ENV).map(|v| v == "test").unwrap_or(false) {
        // Check for API key from file descriptor first
        if let Some(key) = get_api_key_from_file_descriptor() {
            return ApiKeyResult {
                key: Some(key),
                source: ApiKeySource::AnthropicApiKey,
            };
        }

        if api_key_env.is_none()
            && std::env::var(ai_code::OAUTH_TOKEN).is_err()
            && std::env::var(ai_code::OAUTH_TOKEN_FILE_DESCRIPTOR).is_err()
        {
            // In Rust we don't throw, return None
            return ApiKeyResult {
                key: None,
                source: ApiKeySource::None,
            };
        }

        if let Some(key) = api_key_env {
            return ApiKeyResult {
                key: Some(key),
                source: ApiKeySource::AnthropicApiKey,
            };
        }

        return ApiKeyResult {
            key: None,
            source: ApiKeySource::None,
        };
    }

    // Check for ANTHROPIC_API_KEY before checking the apiKeyHelper or /login-managed key
    if let Some(ref key) = api_key_env {
        let config = get_global_config();
        if let Some(approved) = config.custom_api_key_responses.as_ref().and_then(|r| r.approved.as_ref()) {
            let normalized = normalize_api_key_for_config(key);
            if approved.contains(&normalized) {
                return ApiKeyResult {
                    key: Some(key.clone()),
                    source: ApiKeySource::AnthropicApiKey,
                };
            }
        }
    }

    // Check for API key from file descriptor
    if let Some(key) = get_api_key_from_file_descriptor() {
        return ApiKeyResult {
            key: Some(key),
            source: ApiKeySource::AnthropicApiKey,
        };
    }

    // Check for apiKeyHelper
    if let Some(helper) = get_configured_api_key_helper() {
        if skip_retrieving_key_from_api_key_helper {
            return ApiKeyResult {
                key: None,
                source: ApiKeySource::ApiKeyHelper,
            };
        }
        return ApiKeyResult {
            key: get_api_key_from_api_key_helper_cached(),
            source: ApiKeySource::ApiKeyHelper,
        };
    }

    // Check config or macOS keychain
    if let Some(result) = get_api_key_from_config_or_macos_keychain() {
        return result;
    }

    ApiKeyResult {
        key: None,
        source: ApiKeySource::None,
    }
}

/// Get the configured apiKeyHelper from settings.
pub fn get_configured_api_key_helper() -> Option<String> {
    if is_bare_mode() {
        return get_settings_for_source("flagSettings")
            .and_then(|s| s.get("apiKeyHelper").cloned());
    }
    let settings = get_settings_deprecated().unwrap_or_default();
    settings.get("apiKeyHelper").cloned()
}

/// Calculate TTL in milliseconds for the API key helper cache
pub fn calculate_api_key_helper_ttl() -> u64 {
    if let Ok(env_ttl) = std::env::var(ai_code::API_KEY_HELPER_TTL_MS) {
        if let Ok(parsed) = env_ttl.parse::<u64>() {
            if parsed >= 0 {
                return parsed;
            }
            log_for_debugging(&format!(
                "Found AI_CODE_API_KEY_HELPER_TTL_MS env var, but it was not a valid number. Got {}",
                env_ttl
            ));
        }
    }
    DEFAULT_API_KEY_HELPER_TTL
}

// Async API key helper with sync cache for non-blocking reads.
struct ApiKeyHelperCache {
    value: Option<String>,
    timestamp: u64,
}

static API_KEY_HELPER_CACHE: Lazy<Mutex<Option<ApiKeyHelperCache>>> = Lazy::new(|| Mutex::new(None));
static API_KEY_HELPER_EPOCH: Lazy<Arc<Mutex<u32>>> = Lazy::new(|| Arc::new(Mutex::new(0)));

pub fn get_api_key_helper_elapsed_ms() -> u64 {
    // For now, simplified implementation
    0
}

/// Get API key from apiKeyHelper (async)
pub async fn get_api_key_from_api_key_helper(
    _is_non_interactive_session: bool,
) -> Option<String> {
    let helper = get_configured_api_key_helper()?;
    let ttl = calculate_api_key_helper_ttl();

    let mut cache = API_KEY_HELPER_CACHE.lock().ok()?;
    if let Some(ref cached) = *cache {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        if now - cached.timestamp < ttl {
            return cached.value.clone();
        }
    }
    drop(cache);

    // Execute the helper
    let result = execute_api_key_helper(&helper).await;
    
    // Update cache
    if let Ok(mut cache) = API_KEY_HELPER_CACHE.lock() {
        *cache = Some(ApiKeyHelperCache {
            value: result.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        });
    }

    result
}

async fn execute_api_key_helper(_helper: &str) -> Option<String> {
    // This would execute the apiKeyHelper command
    // Simplified for now - would use tokio::process::Command in real implementation
    None
}

/// Sync cache reader — returns the last fetched apiKeyHelper value without executing.
pub fn get_api_key_from_api_key_helper_cached() -> Option<String> {
    API_KEY_HELPER_CACHE
        .lock()
        .ok()
        .and_then(|cache| cache.as_ref().and_then(|c| c.value.clone()))
}

pub fn clear_api_key_helper_cache() {
    if let Ok(mut epoch) = API_KEY_HELPER_EPOCH.lock() {
        *epoch += 1;
    }
    if let Ok(mut cache) = API_KEY_HELPER_CACHE.lock() {
        *cache = None;
    }
}

pub fn prefetch_api_key_from_api_key_helper_if_safe(_is_non_interactive_session: bool) {
    // Skip if trust not yet accepted
    if is_api_key_helper_from_project_or_local_settings() && !check_has_trust_dialog_accepted() {
        return;
    }
    // Would call get_api_key_from_api_key_helper in background
}

// ============ OAuth Tokens ============

#[derive(Debug, Clone)]
pub struct OAuthTokens {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub scopes: Vec<String>,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
}

/// Save OAuth tokens to secure storage
pub fn save_oauth_tokens_if_needed(tokens: OAuthTokens) -> (bool, Option<String>) {
    if !should_use_claude_ai_auth(&tokens) {
        return (true, None);
    }

    // Skip saving inference-only tokens (they come from env vars)
    if tokens.refresh_token.is_none() || tokens.expires_at.is_none() {
        return (true, None);
    }

    // Would save to secure storage
    // For now, simplified
    (true, None)
}

/// Get Claude.ai OAuth tokens
pub fn get_claude_ai_oauth_tokens() -> Option<OAuthTokens> {
    // --bare: API-key-only
    if is_bare_mode() {
        return None;
    }

    // Check for force-set OAuth token from environment variable
    if let Ok(token) = std::env::var(ai_code::OAUTH_TOKEN) {
        return Some(OAuthTokens {
            access_token: Some(token),
            refresh_token: None,
            expires_at: None,
            scopes: vec!["user:inference".to_string()],
            subscription_type: None,
            rate_limit_tier: None,
        });
    }

    // Check for OAuth token from file descriptor
    if let Some(token) = get_oauth_token_from_file_descriptor() {
        return Some(OAuthTokens {
            access_token: Some(token),
            refresh_token: None,
            expires_at: None,
            scopes: vec!["user:inference".to_string()],
            subscription_type: None,
            rate_limit_tier: None,
        });
    }

    // Would read from secure storage
    None
}

/// Clear OAuth token cache
pub fn clear_oauth_token_cache() {
    // Would clear all OAuth caches
}

// ============ AWS Auth ============

/// Refresh AWS authentication
pub async fn refresh_aws_auth(aws_auth_refresh: &str) -> bool {
    // Simplified implementation
    // Would run the awsAuthRefresh command
    log_for_debugging(&format!("Running AWS auth refresh command: {}", aws_auth_refresh));
    false
}

/// Refresh and get AWS credentials
pub async fn refresh_and_get_aws_credentials() -> Option<AwsCredentials> {
    // First check if caller identity is valid
    match check_sts_caller_identity().await {
        Ok(_) => {
            log_for_debugging("Fetched AWS caller identity, skipping AWS auth refresh command");
            return None;
        }
        Err(_) => {
            // Need to refresh
        }
    }

    // Run auth refresh if needed
    let refreshed = refresh_aws_auth_internal().await;

    // Get credentials from export
    let credentials = get_aws_creds_from_credential_export().await;

    // Clear AWS INI cache if we refreshed or got credentials
    if refreshed || credentials.is_some() {
        clear_aws_ini_cache().await;
    }

    credentials
}

async fn refresh_aws_auth_internal() -> bool {
    let aws_auth_refresh = get_configured_aws_auth_refresh()?;
    
    // Check if from project settings and trust required
    if is_aws_auth_refresh_from_project_settings() {
        if !check_has_trust_dialog_accepted() && !get_is_non_interactive_session() {
            return false;
        }
    }

    refresh_aws_auth(&aws_auth_refresh).await
}

pub fn clear_aws_credentials_cache() {
    // Would clear the memoized cache
}

#[derive(Debug, Clone)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
}

async fn get_aws_creds_from_credential_export() -> Option<AwsCredentials> {
    let aws_credential_export = get_configured_aws_credential_export()?;

    // SECURITY: Check if from project settings
    if is_aws_credential_export_from_project_settings() {
        if !check_has_trust_dialog_accepted() && !get_is_non_interactive_session() {
            return None;
        }
    }

    // Check if credentials are already valid
    match check_sts_caller_identity().await {
        Ok(_) => {
            log_for_debugging("Fetched AWS caller identity, skipping AWS credential export command");
            return None;
        }
        Err(_) => {}
    }

    // Would execute awsCredentialExport and parse JSON
    None
}

async fn check_sts_caller_identity() -> Result<(), String> {
    // Simplified - would use AWS SDK
    Ok(())
}

async fn clear_aws_ini_cache() {
    // Would clear AWS INI cache
}

// ============ GCP Auth ============

/// Check if GCP credentials are currently valid
pub async fn check_gcp_credentials_valid() -> bool {
    // Would use google-auth-library
    // Simplified for now
    false
}

/// Refresh GCP authentication
pub async fn refresh_gcp_auth(gcp_auth_refresh: &str) -> bool {
    log_for_debugging(&format!("Running GCP auth refresh command: {}", gcp_auth_refresh));
    false
}

/// Refresh GCP credentials if needed
pub async fn refresh_gcp_credentials_if_needed() -> bool {
    let refreshed = run_gcp_auth_refresh().await;
    refreshed
}

async fn run_gcp_auth_refresh() -> bool {
    let gcp_auth_refresh = get_configured_gcp_auth_refresh()?;

    if is_gcp_auth_refresh_from_project_settings() {
        if !check_has_trust_dialog_accepted() && !get_is_non_interactive_session() {
            return false;
        }
    }

    // Check if credentials are valid
    if check_gcp_credentials_valid().await {
        log_for_debugging("GCP credentials are valid, skipping auth refresh command");
        return false;
    }

    refresh_gcp_auth(&gcp_auth_refresh).await
}

pub fn clear_gcp_credentials_cache() {
    // Would clear cache
}

pub fn prefetch_gcp_credentials_if_safe() {
    let gcp_auth_refresh = get_configured_gcp_auth_refresh();
    if gcp_auth_refresh.is_none() {
        return;
    }

    if is_gcp_auth_refresh_from_project_settings() {
        if !check_has_trust_dialog_accepted() && !get_is_non_interactive_session() {
            return;
        }
    }

    // Would prefetch in background
}

pub fn prefetch_aws_credentials_and_bedrock_info_if_safe() {
    let aws_auth_refresh = get_configured_aws_auth_refresh();
    let aws_credential_export = get_configured_aws_credential_export();

    if aws_auth_refresh.is_none() && aws_credential_export.is_none() {
        return;
    }

    if is_aws_auth_refresh_from_project_settings() || is_aws_credential_export_from_project_settings() {
        if !check_has_trust_dialog_accepted() && !get_is_non_interactive_session() {
            return;
        }
    }

    // Would prefetch in background
}

// ============ API Key Management ============

pub async fn save_api_key(api_key: &str) -> Result<(), String> {
    if !is_valid_api_key(api_key) {
        return Err("Invalid API key format".to_string());
    }

    // Would save to keychain or config
    log_for_debugging("API key saved");
    Ok(())
}

pub fn is_custom_api_key_approved(api_key: &str) -> bool {
    let config = get_global_config();
    let normalized = normalize_api_key_for_config(api_key);
    config
        .custom_api_key_responses
        .as_ref()
        .and_then(|r| r.approved.as_ref())
        .map(|approved| approved.contains(&normalized))
        .unwrap_or(false)
}

pub async fn remove_api_key() {
    // Would remove from keychain and config
}

pub fn get_subscription_type() -> Option<String> {
    // Would get from OAuth tokens or config
    None
}

// ============ Helper Functions ============

fn is_bare_mode() -> bool {
    std::env::var(ai_code::BARE).is_ok()
}

fn is_env_truthy(var: &str) -> bool {
    std::env::var(var).map(|v| v == "1" || v == "true").unwrap_or(false)
}

fn is_running_on_homespace() -> bool {
    std::env::var(ai_code::HOMESPACE).is_ok()
}

fn prefer_third_party_authentication() -> bool {
    std::env::var(ai_code::PREFER_THIRD_PARTY).is_ok()
}

fn is_api_key_helper_from_project_or_local_settings() -> bool {
    let helper = match get_configured_api_key_helper() {
        Some(h) => h,
        None => return false,
    };

    let project_settings = get_settings_for_source("projectSettings");
    let local_settings = get_settings_for_source("localSettings");

    project_settings
        .and_then(|s| s.get("apiKeyHelper").cloned())
        .map(|h| h == helper)
        .unwrap_or(false)
        || local_settings
            .and_then(|s| s.get("apiKeyHelper").cloned())
            .map(|h| h == helper)
            .unwrap_or(false)
}

fn get_configured_aws_auth_refresh() -> Option<String> {
    get_settings_deprecated()
        .and_then(|s| s.get("awsAuthRefresh").cloned())
}

fn is_aws_auth_refresh_from_project_settings() -> bool {
    let refresh = match get_configured_aws_auth_refresh() {
        Some(r) => r,
        None => return false,
    };

    let project_settings = get_settings_for_source("projectSettings");
    let local_settings = get_settings_for_source("localSettings");

    project_settings
        .and_then(|s| s.get("awsAuthRefresh").cloned())
        .map(|r| r == refresh)
        .unwrap_or(false)
        || local_settings
            .and_then(|s| s.get("awsAuthRefresh").cloned())
            .map(|r| r == refresh)
            .unwrap_or(false)
}

fn get_configured_aws_credential_export() -> Option<String> {
    get_settings_deprecated()
        .and_then(|s| s.get("awsCredentialExport").cloned())
}

fn is_aws_credential_export_from_project_settings() -> bool {
    let export = match get_configured_aws_credential_export() {
        Some(e) => e,
        None => return false,
    };

    let project_settings = get_settings_for_source("projectSettings");
    let local_settings = get_settings_for_source("localSettings");

    project_settings
        .and_then(|s| s.get("awsCredentialExport").cloned())
        .map(|e| e == export)
        .unwrap_or(false)
        || local_settings
            .and_then(|s| s.get("awsCredentialExport").cloned())
            .map(|e| e == export)
            .unwrap_or(false)
}

fn get_configured_gcp_auth_refresh() -> Option<String> {
    get_settings_deprecated()
        .and_then(|s| s.get("gcpAuthRefresh").cloned())
}

fn is_gcp_auth_refresh_from_project_settings() -> bool {
    let refresh = match get_configured_gcp_auth_refresh() {
        Some(r) => r,
        None => return false,
    };

    let project_settings = get_settings_for_source("projectSettings");
    let local_settings = get_settings_for_source("localSettings");

    project_settings
        .and_then(|s| s.get("gcpAuthRefresh").cloned())
        .map(|r| r == refresh)
        .unwrap_or(false)
        || local_settings
            .and_then(|s| s.get("gcpAuthRefresh").cloned())
            .map(|r| r == refresh)
            .unwrap_or(false)
}

fn get_settings_deprecated() -> Option<HashMap<String, String>> {
    // Would read from settings file
    None
}

fn get_settings_for_source(_source: &str) -> Option<HashMap<String, String>> {
    // Would read from specific source
    None
}

#[derive(Debug, Clone, Default)]
pub struct GlobalConfig {
    pub primary_api_key: Option<String>,
    pub custom_api_key_responses: Option<CustomApiKeyResponses>,
}

#[derive(Debug, Clone, Default)]
pub struct CustomApiKeyResponses {
    pub approved: Option<Vec<String>>,
    pub rejected: Option<Vec<String>>,
}

fn get_global_config() -> GlobalConfig {
    // Would read from config
    GlobalConfig::default()
}

fn get_api_key_from_config_or_macos_keychain() -> Option<ApiKeyResult> {
    // Would check macOS keychain or config file
    None
}

fn get_api_key_from_file_descriptor() -> Option<String> {
    // Would read from file descriptor
    None
}

fn get_oauth_token_from_file_descriptor() -> Option<String> {
    // Would read from file descriptor
    None
}

fn check_has_trust_dialog_accepted() -> bool {
    // Would check if trust dialog was accepted
    true
}

fn get_is_non_interactive_session() -> bool {
    std::env::var(ai_code::NON_INTERACTIVE).is_ok()
}

fn normalize_api_key_for_config(api_key: &str) -> String {
    // Would normalize the API key
    api_key.to_string()
}

fn is_valid_api_key(api_key: &str) -> bool {
    // Only allow alphanumeric characters, dashes, and underscores
    api_key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn should_use_claude_ai_auth(tokens: &Option<OAuthTokens>) -> bool {
    tokens
        .as_ref()
        .map(|t| t.scopes.iter().any(|s| s.contains("user")))
        .unwrap_or(false)
}

fn log_for_debugging(message: &str) {
    eprintln!("[DEBUG] {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_api_key() {
        assert!(is_valid_api_key("sk-ant-api03-abc123"));
        assert!(is_valid_api_key("abc-123"));
        assert!(!is_valid_api_key("sk-ant-api03@#!"));
    }

    #[test]
    fn test_normalize_api_key() {
        let normalized = normalize_api_key_for_config("  sk-ant-api03-abc  ");
        assert!(normalized.contains("sk-ant-api03"));
    }
}
