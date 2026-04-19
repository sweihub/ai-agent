// Source: /data/home/swei/claudecode/openclaudecode/src/constants/oauth.ts
//! OAuth configuration constants and endpoints.
//!
//! Production OAuth is configured by default, with support for staging and local overrides
//! via environment variables.

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref OAUTH_CONFIG: Mutex<OAuthConfig> = Mutex::new(get_oauth_config_inner());
}

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub base_api_url: String,
    pub console_authorize_url: String,
    pub claude_ai_authorize_url: String,
    pub claude_ai_origin: String,
    pub token_url: String,
    pub api_key_url: String,
    pub roles_url: String,
    pub console_success_url: String,
    pub claudeai_success_url: String,
    pub manual_redirect_url: String,
    pub client_id: String,
    pub oauth_file_suffix: String,
    pub mcp_proxy_url: String,
    pub mcp_proxy_path: String,
}

fn get_oauth_config_inner() -> OAuthConfig {
    // Support custom OAuth URL override for FedStart/PubSec deployments
    if let Some(oauth_base_url) = std::env::var("AI_CODE_CUSTOM_OAUTH_URL").ok().filter(|s| !s.is_empty()) {
        let base = oauth_base_url.trim_end_matches('/');
        // In Rust we trust the caller; TypeScript validates against ALLOWED_OAUTH_BASE_URLS
        return OAuthConfig {
            base_api_url: base.to_string(),
            console_authorize_url: format!("{base}/oauth/authorize"),
            claude_ai_authorize_url: format!("{base}/oauth/authorize"),
            claude_ai_origin: base.to_string(),
            token_url: format!("{base}/v1/oauth/token"),
            api_key_url: format!("{base}/api/oauth/claude_cli/create_api_key"),
            roles_url: format!("{base}/api/oauth/claude_cli/roles"),
            console_success_url: format!("{base}/oauth/code/success?app=claude-code"),
            claudeai_success_url: format!("{base}/oauth/code/success?app=claude-code"),
            manual_redirect_url: format!("{base}/oauth/code/callback"),
            oauth_file_suffix: "-custom-oauth".to_string(),
            ..OAuthConfig::default()
        };
    }

    // Staging config
    if std::env::var("AI_CODE_OAUTH_STAGING").ok().is_some() {
        return OAuthConfig {
            base_api_url: "https://api-staging.anthropic.com".to_string(),
            console_authorize_url: "https://platform.staging.ant.dev/oauth/authorize".to_string(),
            claude_ai_authorize_url: "https://claude-ai.staging.ant.dev/oauth/authorize".to_string(),
            claude_ai_origin: "https://claude-ai.staging.ant.dev".to_string(),
            token_url: "https://platform.staging.ant.dev/v1/oauth/token".to_string(),
            api_key_url: "https://api-staging.anthropic.com/api/oauth/claude_cli/create_api_key".to_string(),
            roles_url: "https://api-staging.anthropic.com/api/oauth/claude_cli/roles".to_string(),
            console_success_url: "https://platform.staging.ant.dev/buy_credits?returnUrl=/oauth/code/success%3Fapp%3Dclaude-code".to_string(),
            claudeai_success_url: "https://platform.staging.ant.dev/oauth/code/success?app=claude-code".to_string(),
            manual_redirect_url: "https://platform.staging.ant.dev/oauth/code/callback".to_string(),
            client_id: "22422756-60c9-4084-8eb7-27705fd5cf9a".to_string(),
            oauth_file_suffix: "-staging-oauth".to_string(),
            mcp_proxy_url: "https://mcp-proxy-staging.anthropic.com".to_string(),
            mcp_proxy_path: "/v1/mcp/{server_id}".to_string(),
        };
    }

    // Local dev config
    if std::env::var("AI_CODE_OAUTH_LOCAL").ok().is_some() {
        let api = std::env::var("AI_LOCAL_OAUTH_API_BASE")
            .ok()
            .map(|s| s.trim_end_matches('/').to_string())
            .unwrap_or_else(|| "http://localhost:8000".to_string());
        let apps = std::env::var("AI_LOCAL_OAUTH_APPS_BASE")
            .ok()
            .map(|s| s.trim_end_matches('/').to_string())
            .unwrap_or_else(|| "http://localhost:4000".to_string());
        let console_base = std::env::var("AI_LOCAL_OAUTH_CONSOLE_BASE")
            .ok()
            .map(|s| s.trim_end_matches('/').to_string())
            .unwrap_or_else(|| "http://localhost:3000".to_string());
        return OAuthConfig {
            base_api_url: api.clone(),
            console_authorize_url: format!("{console_base}/oauth/authorize"),
            claude_ai_authorize_url: format!("{apps}/oauth/authorize"),
            claude_ai_origin: apps.clone(),
            token_url: format!("{api}/v1/oauth/token"),
            api_key_url: format!("{api}/api/oauth/claude_cli/create_api_key"),
            roles_url: format!("{api}/api/oauth/claude_cli/roles"),
            console_success_url: format!("{console_base}/buy_credits?returnUrl=/oauth/code/success%3Fapp%3Dclaude-code"),
            claudeai_success_url: format!("{console_base}/oauth/code/success?app=claude-code"),
            manual_redirect_url: format!("{console_base}/oauth/code/callback"),
            client_id: "22422756-60c9-4084-8eb7-27705fd5cf9a".to_string(),
            oauth_file_suffix: "-local-oauth".to_string(),
            mcp_proxy_url: "http://localhost:8205".to_string(),
            mcp_proxy_path: "/v1/toolbox/shttp/mcp/{server_id}".to_string(),
        };
    }

    // Production config
    OAuthConfig::default()
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            base_api_url: "https://api.anthropic.com".to_string(),
            console_authorize_url: "https://platform.claude.com/oauth/authorize".to_string(),
            claude_ai_authorize_url: "https://claude.com/cai/oauth/authorize".to_string(),
            claude_ai_origin: "https://claude.ai".to_string(),
            token_url: "https://platform.claude.com/v1/oauth/token".to_string(),
            api_key_url: "https://api.anthropic.com/api/oauth/claude_cli/create_api_key".to_string(),
            roles_url: "https://api.anthropic.com/api/oauth/claude_cli/roles".to_string(),
            console_success_url: "https://platform.claude.com/buy_credits?returnUrl=/oauth/code/success%3Fapp%3Dclaude-code".to_string(),
            claudeai_success_url: "https://platform.claude.com/oauth/code/success?app=claude-code".to_string(),
            manual_redirect_url: "https://platform.claude.com/oauth/code/callback".to_string(),
            client_id: "9d1c250a-e61b-44d9-88ed-5944d988ed5944d9".to_string(),
            oauth_file_suffix: "".to_string(),
            mcp_proxy_url: "https://mcp-proxy.anthropic.com".to_string(),
            mcp_proxy_path: "/v1/mcp/{server_id}".to_string(),
        }
    }
}

/// Client ID metadata URL for MCP OAuth (CIMD / SEP-991).
pub const MCP_CLIENT_METADATA_URL: &str = "https://claude.ai/oauth/claude-code-client-metadata";

/// Get the OAuth configuration.
pub fn get_oauth_config() -> OAuthConfig {
    OAUTH_CONFIG.lock().unwrap().clone()
}

/// Set the OAuth configuration (for testing).
pub fn set_oauth_config(config: OAuthConfig) {
    let mut cfg = OAUTH_CONFIG.lock().unwrap();
    *cfg = config;
}

/// Get the OAuth client ID, with env override.
pub fn get_client_id() -> String {
    std::env::var("AI_CODE_OAUTH_CLIENT_ID")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| get_oauth_config().client_id)
}
