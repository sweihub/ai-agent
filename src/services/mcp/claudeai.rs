// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/claudeai.ts
//! Claude.ai MCP server configurations fetching

use std::collections::HashMap;
use std::sync::Mutex;

use crate::session_history::get_claude_ai_oauth_tokens;
use crate::utils::http::get_user_agent;

/// Fetch timeout in milliseconds
const FETCH_TIMEOUT_MS: u64 = 5000;
/// MCP servers beta header
const MCP_SERVERS_BETA_HEADER: &str = "mcp-servers-2025-12-04";

/// Claude.ai MCP server
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeAiMcpServer {
    #[serde(rename = "type")]
    pub server_type: String,
    pub id: String,
    pub display_name: String,
    pub url: String,
    pub created_at: String,
}

/// Claude.ai MCP servers response
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeAiMcpServersResponse {
    pub data: Vec<ClaudeAiMcpServer>,
    pub has_more: bool,
    #[serde(default)]
    pub next_page: Option<String>,
}

/// Cache for MCP server configs (memoized for session lifetime)
static CONFIGS_CACHE: Mutex<Option<HashMap<String, crate::services::mcp::types::ScopedMcpServerConfig>>> =
    Mutex::new(None);

/// Check if the env var is defined and falsy (empty/0/false/no/off)
fn is_env_defined_falsy(env_var: Option<String>) -> bool {
    match env_var {
        None => false,
        Some(val) => {
            let trimmed = val.trim().to_lowercase();
            trimmed.is_empty() || ["0", "false", "no", "off"].contains(&trimmed.as_str())
        }
    }
}

/// Check if Claude.ai MCP servers feature is enabled via environment
/// Uses isEnvDefinedFalsy logic from TS: enabled unless explicitly disabled
fn is_enabled_via_env() -> bool {
    !is_env_defined_falsy(std::env::var("AI_CODE_ENABLE_CLAUDEAI_MCP_SERVERS").ok())
}

/// Check if user has OAuth access token with user:mcp_servers scope
fn has_required_oauth_scope() -> bool {
    if let Some(tokens) = get_claude_ai_oauth_tokens() {
        tokens.scopes.iter().any(|s| s == "user:mcp_servers")
    } else {
        false
    }
}

/// Get the base OAuth API URL
fn get_base_api_url() -> String {
    // In TS this is getOauthConfig().BASE_API_URL
    // For SDK, use the known value
    std::env::var("AI_CODE_BASE_API_URL")
        .unwrap_or_else(|_| "https://api.anthropic.com".to_string())
}

/// Fetch MCP server configs from Claude.ai API
/// Results are memoized for the session lifetime (fetch once per CLI session)
pub async fn fetch_claudeai_mcp_configs_if_eligible() -> HashMap<String, crate::services::mcp::types::ScopedMcpServerConfig> {
    // Check if disabled via env var (isEnvDefinedFalsy)
    if is_env_defined_falsy(std::env::var("AI_CODE_ENABLE_CLAUDEAI_MCP_SERVERS").ok()) {
        log::debug!("[claudeai-mcp] Disabled via env var");
        return HashMap::new();
    }

    // Check for OAuth token with user:mcp_servers scope
    if !has_required_oauth_scope() {
        log::debug!("[claudeai-mcp] No access token or missing scope");
        return HashMap::new();
    }

    // Check cache first (memoized)
    {
        let cache = CONFIGS_CACHE.lock().unwrap();
        if let Some(ref configs) = *cache {
            return configs.clone();
        }
    }

    // Fetch from API
    let base_url = get_base_api_url();
    let url = format!("{}/v1/mcp_servers?limit=1000", base_url);

    log::debug!("[claudeai-mcp] Fetching from {}", url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(FETCH_TIMEOUT_MS))
        .build();

    let Ok(client) = client else {
        return HashMap::new();
    };

    let access_token = get_claude_ai_oauth_tokens()
        .map(|t| t.access_token)
        .unwrap_or_default();

    let result = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("anthropic-beta", MCP_SERVERS_BETA_HEADER)
        .header("anthropic-version", "2023-06-01")
        .header("User-Agent", get_user_agent())
        .send()
        .await;

    match result {
        Ok(response) => {
            if let Ok(data) = response.json::<ClaudeAiMcpServersResponse>().await {
                let configs = convert_to_scoped_configs(&data);
                let mut cache = CONFIGS_CACHE.lock().unwrap();
                *cache = Some(configs.clone());
                return configs;
            }
        }
        Err(e) => {
            log::debug!("[claudeai-mcp] Failed to fetch: {}", e);
        }
    }

    HashMap::new()
}

/// Convert API response to scoped MCP server configs
/// Handles name collision by appending (2), (3), etc. suffixes
fn convert_to_scoped_configs(response: &ClaudeAiMcpServersResponse) -> HashMap<String, crate::services::mcp::types::ScopedMcpServerConfig> {
    use crate::services::mcp::types::*;
    use crate::services::mcp::normalize_name_for_mcp;

    let mut configs = HashMap::new();
    let mut used_normalized_names: HashSet<String> = HashSet::new();

    for server in &response.data {
        // TS: `const baseName = \`claude.ai ${server.display_name}\``
        let base_name = format!("claude.ai {}", server.display_name);

        // Try without suffix first, then increment until we find an unused normalized name
        let mut final_name = base_name.clone();
        let mut final_normalized = normalize_name_for_mcp(&final_name);
        let mut count = 1;
        while used_normalized_names.contains(&final_normalized) {
            count += 1;
            final_name = format!("{} ({})", base_name, count);
            final_normalized = normalize_name_for_mcp(&final_name);
        }
        used_normalized_names.insert(final_normalized.clone());

        let config = ScopedMcpServerConfig {
            config: McpServerConfig::ClaudeAiProxy(McpClaudeAiProxyServerConfig {
                config_type: "claudeai-proxy".to_string(),
                url: server.url.clone(),
                id: server.id.clone(),
            }),
            scope: ConfigScope::ClaudeAi,
            plugin_source: Some(format!("claude.ai:{}", server.id)),
        };

        configs.insert(final_name, config);
    }

    configs
}

/// Clear the cached configs (for testing)
pub fn clear_configs_cache() {
    let mut cache = CONFIGS_CACHE.lock().unwrap();
    *cache = None;
}

use std::collections::HashSet;