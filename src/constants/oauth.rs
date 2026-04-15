// Source: /data/home/swei/claudecode/openclaudecode/src/constants/oauth.ts
//! OAuth configuration constants

use once_cell::sync::Lazy;
use std::env;

pub const CLAUDE_AI_INFERENCE_SCOPE: &str = "user:inference";
pub const CLAUDE_AI_PROFILE_SCOPE: &str = "user:profile";
const CONSOLE_SCOPE: &str = "org:create_api_key";
pub const OAUTH_BETA_HEADER: &str = "oauth-2025-04-20";

pub const CONSOLE_OAUTH_SCOPES: &[&str] = &[CONSOLE_SCOPE, CLAUDE_AI_PROFILE_SCOPE];

pub const CLAUDE_AI_OAUTH_SCOPES: &[&str] = &[
    CLAUDE_AI_PROFILE_SCOPE,
    CLAUDE_AI_INFERENCE_SCOPE,
    "user:sessions:claude_code",
    "user:mcp_servers",
    "user:file_upload",
];

pub fn get_all_oauth_scopes() -> Vec<&'static str> {
    let mut scopes: Vec<&str> = CONSOLE_OAUTH_SCOPES.to_vec();
    for scope in CLAUDE_AI_OAUTH_SCOPES {
        if !scopes.contains(scope) {
            scopes.push(scope);
        }
    }
    scopes
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OauthConfigType {
    Prod,
    Staging,
    Local,
}

fn get_oauth_config_type() -> OauthConfigType {
    let user_type = env::var("USER_TYPE").unwrap_or_default();
    if user_type == "ant" {
        let use_local = env::var("USE_LOCAL_OAUTH")
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .unwrap_or(false);
        if use_local {
            return OauthConfigType::Local;
        }
        let use_staging = env::var("USE_STAGING_OAUTH")
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .unwrap_or(false);
        if use_staging {
            return OauthConfigType::Staging;
        }
    }
    OauthConfigType::Prod
}

pub fn file_suffix_for_oauth_config() -> String {
    if env::var("AI_CODE_CUSTOM_OAUTH_URL").is_ok() {
        return "-custom-oauth".to_string();
    }
    match get_oauth_config_type() {
        OauthConfigType::Local => "-local-oauth".to_string(),
        OauthConfigType::Staging => "-staging-oauth".to_string(),
        OauthConfigType::Prod => "".to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct OauthConfig {
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

pub const MCP_CLIENT_METADATA_URL: &str = "https://claude.ai/oauth/claude-code-client-metadata";

const ALLOWED_OAUTH_BASE_URLS: &[&str] = &[
    "https://beacon.claude-ai.staging.ant.dev",
    "https://claude.fedstart.com",
    "https://claude-staging.fedstart.com",
];

fn get_local_oauth_config() -> OauthConfig {
    let api = env::var("CLAUDE_LOCAL_OAUTH_API_BASE")
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let apps = env::var("CLAUDE_LOCAL_OAUTH_APPS_BASE")
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|_| "http://localhost:4000".to_string());
    let console_base = env::var("CLAUDE_LOCAL_OAUTH_CONSOLE_BASE")
        .map(|s| s.trim_end_matches('/').to_string())
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    OauthConfig {
        base_api_url: api.clone(),
        console_authorize_url: format!("{}/oauth/authorize", console_base),
        claude_ai_authorize_url: format!("{}/oauth/authorize", apps),
        claude_ai_origin: apps,
        token_url: format!("{}/v1/oauth/token", api),
        api_key_url: format!("{}/api/oauth/claude_cli/create_api_key", api),
        roles_url: format!("{}/api/oauth/claude_cli/roles", api),
        console_success_url: format!(
            "{}/buy_credits?returnUrl=/oauth/code/success%3Fapp%3Dclaude-code",
            console_base
        ),
        claudeai_success_url: format!("{}/oauth/code/success?app=claude-code", console_base),
        manual_redirect_url: format!("{}/oauth/code/callback", console_base),
        client_id: "22422756-60c9-4084-8eb7-27705fd5cf9a".to_string(),
        oauth_file_suffix: "-local-oauth".to_string(),
        mcp_proxy_url: "http://localhost:8205".to_string(),
        mcp_proxy_path: "/v1/toolbox/shttp/mcp/{server_id}".to_string(),
    }
}

static PROD_OAUTH_CONFIG: Lazy<OauthConfig> = Lazy::new(|| OauthConfig {
    base_api_url: "https://api.anthropic.com".to_string(),
    console_authorize_url: "https://platform.claude.com/oauth/authorize".to_string(),
    claude_ai_authorize_url: "https://claude.com/cai/oauth/authorize".to_string(),
    claude_ai_origin: "https://claude.ai".to_string(),
    token_url: "https://platform.claude.com/v1/oauth/token".to_string(),
    api_key_url: "https://api.anthropic.com/api/oauth/claude_cli/create_api_key".to_string(),
    roles_url: "https://api.anthropic.com/api/oauth/claude_cli/roles".to_string(),
    console_success_url:
        "https://platform.claude.com/buy_credits?returnUrl=/oauth/code/success%3Fapp%3Dclaude-code"
            .to_string(),
    claudeai_success_url: "https://platform.claude.com/oauth/code/success?app=claude-code"
        .to_string(),
    manual_redirect_url: "https://platform.claude.com/oauth/code/callback".to_string(),
    client_id: "9d1c250a-e61b-44d9-88ed-5944d1962f5e".to_string(),
    oauth_file_suffix: "".to_string(),
    mcp_proxy_url: "https://mcp-proxy.anthropic.com".to_string(),
    mcp_proxy_path: "/v1/mcp/{server_id}".to_string(),
});

pub fn get_oauth_config() -> OauthConfig {
    let base_config = match get_oauth_config_type() {
        OauthConfigType::Local => get_local_oauth_config(),
        OauthConfigType::Staging => {
            // For staging, check if we're an ant build
            if env::var("USER_TYPE").map(|t| t == "ant").unwrap_or(false) {
                OauthConfig {
                    base_api_url: "https://api-staging.anthropic.com".to_string(),
                    console_authorize_url:
                        "https://platform.staging.ant.dev/oauth/authorize".to_string(),
                    claude_ai_authorize_url:
                        "https://claude-ai.staging.ant.dev/oauth/authorize".to_string(),
                    claude_ai_origin: "https://claude-ai.staging.ant.dev".to_string(),
                    token_url: "https://platform.staging.ant.dev/v1/oauth/token".to_string(),
                    api_key_url:
                        "https://api-staging.anthropic.com/api/oauth/claude_cli/create_api_key"
                            .to_string(),
                    roles_url:
                        "https://api-staging.anthropic.com/api/oauth/claude_cli/roles".to_string(),
                    console_success_url:
                        "https://platform.staging.ant.dev/buy_credits?returnUrl=/oauth/code/success%3Fapp%3Dclaude-code"
                            .to_string(),
                    claudeai_success_url:
                        "https://platform.staging.ant.dev/oauth/code/success?app=claude-code"
                            .to_string(),
                    manual_redirect_url:
                        "https://platform.staging.ant.dev/oauth/code/callback".to_string(),
                    client_id: "22422756-60c9-4084-8eb7-27705fd5cf9a".to_string(),
                    oauth_file_suffix: "-staging-oauth".to_string(),
                    mcp_proxy_url: "https://mcp-proxy-staging.anthropic.com".to_string(),
                    mcp_proxy_path: "/v1/mcp/{server_id}".to_string(),
                }
            } else {
                PROD_OAUTH_CONFIG.clone()
            }
        }
        OauthConfigType::Prod => PROD_OAUTH_CONFIG.clone(),
    };

    let mut config = base_config;

    // Allow overriding all OAuth URLs to point to an approved FedStart deployment
    if let Ok(oauth_base_url) = env::var("AI_CODE_CUSTOM_OAUTH_URL") {
        let base = oauth_base_url.trim_end_matches('/').to_string();
        if !ALLOWED_OAUTH_BASE_URLS.contains(&base.as_str()) {
            panic!("AI_CODE_CUSTOM_OAUTH_URL is not an approved endpoint.");
        }
        config.base_api_url = base.clone();
        config.console_authorize_url = format!("{}/oauth/authorize", base);
        config.claude_ai_authorize_url = format!("{}/oauth/authorize", base);
        config.claude_ai_origin = base.clone();
        config.token_url = format!("{}/v1/oauth/token", base);
        config.api_key_url = format!("{}/api/oauth/claude_cli/create_api_key", base);
        config.roles_url = format!("{}/api/oauth/claude_cli/roles", base);
        config.console_success_url = format!("{}/oauth/code/success?app=claude-code", base);
        config.claudeai_success_url = format!("{}/oauth/code/success?app=claude-code", base);
        config.manual_redirect_url = format!("{}/oauth/code/callback", base);
        config.oauth_file_suffix = "-custom-oauth".to_string();
    }

    // Allow CLIENT_ID override via environment variable
    if let Ok(client_id_override) = env::var("AI_CODE_OAUTH_CLIENT_ID") {
        config.client_id = client_id_override;
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_oauth_scopes() {
        let scopes = get_all_oauth_scopes();
        assert!(scopes.contains(&CONSOLE_SCOPE));
        assert!(scopes.contains(&CLAUDE_AI_INFERENCE_SCOPE));
    }

    #[test]
    fn test_file_suffix_for_oauth_config() {
        let _ = file_suffix_for_oauth_config();
    }
}
