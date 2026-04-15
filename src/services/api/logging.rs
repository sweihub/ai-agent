// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/logging.ts
//! API logging utilities
//! Handles API query, error, and success logging with analytics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global cache strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlobalCacheStrategy {
    ToolBased,
    SystemPrompt,
    None,
}

impl Default for GlobalCacheStrategy {
    fn default() -> Self {
        GlobalCacheStrategy::None
    }
}

/// Log level for API logging
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiLogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for ApiLogLevel {
    fn default() -> Self {
        ApiLogLevel::Info
    }
}

/// API log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLogEntry {
    pub timestamp: String,
    pub level: ApiLogLevel,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ApiLogEntry {
    pub fn new(level: ApiLogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            level,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn debug(message: impl Into<String>) -> Self {
        Self::new(ApiLogLevel::Debug, message)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(ApiLogLevel::Info, message)
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(ApiLogLevel::Warn, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(ApiLogLevel::Error, message)
    }
}

/// Usage statistics from API response
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    #[serde(rename = "cache_read_input_tokens")]
    pub cache_read_input_tokens: Option<i64>,
    #[serde(rename = "cache_creation_input_tokens")]
    pub cache_creation_input_tokens: Option<i64>,
    pub server_tool_use: Option<ServerToolUse>,
    pub service_tier: Option<&'static str>,
    pub cache_creation: Option<CacheCreation>,
    pub inference_geo: Option<&'static str>,
    pub iterations: Option<Vec<serde_json::Value>>,
    pub speed: Option<&'static str>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ServerToolUse {
    pub web_search_requests: i64,
    pub web_fetch_requests: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CacheCreation {
    pub ephemeral_1h_input_tokens: i64,
    pub ephemeral_5m_input_tokens: i64,
}

/// Empty usage struct - zero-initialized usage object
/// Extracted from emptyUsage.ts so that bridge/replBridge can import it
/// without transitively pulling in api/errors.ts → utils/messages.ts → BashTool.tsx
pub const EMPTY_USAGE: ApiUsage = ApiUsage {
    input_tokens: 0,
    cache_creation_input_tokens: Some(0),
    cache_read_input_tokens: Some(0),
    output_tokens: 0,
    server_tool_use: Some(ServerToolUse {
        web_search_requests: 0,
        web_fetch_requests: 0,
    }),
    service_tier: Some("standard"),
    cache_creation: Some(CacheCreation {
        ephemeral_1h_input_tokens: 0,
        ephemeral_5m_input_tokens: 0,
    }),
    inference_geo: Some(""),
    iterations: Some(Vec::new()),
    speed: Some("standard"),
};

/// Known gateway types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KnownGateway {
    Litellm,
    Helicone,
    Portkey,
    CloudflareAiGateway,
    Kong,
    Braintrust,
    Databricks,
}

/// Gateway fingerprints for detecting AI gateways from response headers
fn get_gateway_fingerprints() -> HashMap<&'static str, Vec<&'static str>> {
    let mut fingerprints = HashMap::new();
    fingerprints.insert("litellm", vec!["x-litellm-"]);
    fingerprints.insert("helicone", vec!["helicone-"]);
    fingerprints.insert("portkey", vec!["x-portkey-"]);
    fingerprints.insert("cloudflare-ai-gateway", vec!["cf-aig-"]);
    fingerprints.insert("kong", vec!["x-kong-"]);
    fingerprints.insert("braintrust", vec!["x-bt-"]);
    fingerprints
}

/// Gateway host suffixes for detection
fn get_gateway_host_suffixes() -> HashMap<&'static str, Vec<&'static str>> {
    let mut suffixes = HashMap::new();
    suffixes.insert("databricks", vec![
        ".cloud.databricks.com",
        ".azuredatabricks.net",
        ".gcp.databricks.com",
    ]);
    suffixes
}

/// Detect gateway from response headers or base URL
pub fn detect_gateway(headers: Option<&HashMap<String, String>>, base_url: Option<&str>) -> Option<KnownGateway> {
    // Check headers for gateway fingerprints
    if let Some(hdrs) = headers {
        let fingerprint_map = get_gateway_fingerprints();
        for (key, prefixes) in fingerprint_map {
            for prefix in prefixes {
                for hdr_name in hdrs.keys() {
                    if hdr_name.to_lowercase().starts_with(prefix) {
                        return match key {
                            "litellm" => Some(KnownGateway::Litellm),
                            "helicone" => Some(KnownGateway::Helicone),
                            "portkey" => Some(KnownGateway::Portkey),
                            "cloudflare-ai-gateway" => Some(KnownGateway::CloudflareAiGateway),
                            "kong" => Some(KnownGateway::Kong),
                            "braintrust" => Some(KnownGateway::Braintrust),
                            "databricks" => Some(KnownGateway::Databricks),
                            _ => None,
                        };
                    }
                }
            }
        }
    }

    // Check base URL for gateway host suffixes
    if let Some(url) = base_url {
        if let Ok(parsed) = url::Url::parse(url) {
            let host = parsed.host_str().map(|h| h.to_lowercase()).unwrap_or_default();
            let suffix_map = get_gateway_host_suffixes();
            for (key, suffixes) in suffix_map {
                for suffix in suffixes {
                    if host.ends_with(suffix) {
                        return Some(KnownGateway::Databricks);
                    }
                }
            }
        }
    }

    None
}

/// Get Anthropic environment metadata from environment variables
pub fn get_anthropic_env_metadata() -> serde_json::Value {
    let mut metadata = serde_json::Map::new();

    if let Ok(base_url) = std::env::var("AI_CODE_BASE_URL") {
        metadata.insert("baseUrl".to_string(), serde_json::Value::String(base_url));
    }
    if let Ok(model) = std::env::var("AI_CODE_MODEL") {
        metadata.insert("envModel".to_string(), serde_json::Value::String(model));
    }
    if let Ok(small_fast_model) = std::env::var("AI_CODE_SMALL_FAST_MODEL") {
        metadata.insert("envSmallFastModel".to_string(), serde_json::Value::String(small_fast_model));
    }

    serde_json::Value::Object(metadata)
}

/// Get build age in minutes if BUILD_TIME is set
pub fn get_build_age_minutes() -> Option<i64> {
    // In real implementation, this would check MACRO.BUILD_TIME
    None
}

/// Check if running in non-interactive session
pub fn is_non_interactive_session() -> bool {
    std::env::var("AI_CODE_NON_INTERACTIVE")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get API provider for statsig
pub fn get_api_provider_for_statsig() -> String {
    std::env::var("AI_CODE_PROVIDER")
        .unwrap_or_else(|_| "firstParty".to_string())
}

/// Log API query event
pub fn log_api_query(model: &str, messages_length: usize, temperature: f64, query_source: &str) {
    log::debug!(
        "[API Query] model={}, messages={}, temp={}, source={}",
        model,
        messages_length,
        temperature,
        query_source
    );
}

/// Log API error event
pub fn log_api_error(
    error_message: &str,
    model: &str,
    message_count: usize,
    duration_ms: u64,
    attempt: u32,
    status: Option<u16>,
    error_type: &str,
) {
    log::error!(
        "[API Error] model={}, status={:?}, error={}, attempt={}, duration_ms={}",
        model,
        status,
        error_message,
        attempt,
        duration_ms
    );
}

/// Log API success event
pub fn log_api_success(
    model: &str,
    message_count: usize,
    message_tokens: i64,
    usage: &ApiUsage,
    duration_ms: u64,
    attempt: u32,
    request_id: Option<&str>,
    stop_reason: Option<&str>,
    cost_usd: f64,
    query_source: &str,
) {
    let input_tokens = usage.input_tokens;
    let output_tokens = usage.output_tokens;
    let cached_tokens = usage.cache_read_input_tokens.unwrap_or(0);
    let uncached_tokens = usage.cache_creation_input_tokens.unwrap_or(0);

    log::debug!(
        "[API Success] model={}, input={}, output={}, cached={}, uncached={}, duration_ms={}, attempt={}, reason={:?}, cost=${:.4}",
        model,
        input_tokens,
        output_tokens,
        cached_tokens,
        uncached_tokens,
        duration_ms,
        attempt,
        stop_reason,
        cost_usd
    );
}

/// Global API logger
pub struct ApiLogger {
    enabled: bool,
    min_level: ApiLogLevel,
}

impl ApiLogger {
    pub fn new() -> Self {
        Self {
            enabled: true,
            min_level: ApiLogLevel::Info,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_min_level(&mut self, level: ApiLogLevel) {
        self.min_level = level;
    }

    pub fn log(&self, entry: &ApiLogEntry) {
        if !self.enabled {
            return;
        }

        let level_priority = match entry.level {
            ApiLogLevel::Debug => 0,
            ApiLogLevel::Info => 1,
            ApiLogLevel::Warn => 2,
            ApiLogLevel::Error => 3,
        };

        let min_priority = match self.min_level {
            ApiLogLevel::Debug => 0,
            ApiLogLevel::Info => 1,
            ApiLogLevel::Warn => 2,
            ApiLogLevel::Error => 3,
        };

        if level_priority >= min_priority {
            eprintln!("[API] {:?}: {}", entry.level, entry.message);
        }
    }
}

impl Default for ApiLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_log_entry_creation() {
        let entry = ApiLogEntry::info("test message");
        assert_eq!(entry.level, ApiLogLevel::Info);
        assert_eq!(entry.message, "test message");
        assert!(entry.details.is_none());
    }

    #[test]
    fn test_api_log_entry_with_details() {
        let entry = ApiLogEntry::info("test").with_details(serde_json::json!({"key": "value"}));
        assert!(entry.details.is_some());
    }
}
