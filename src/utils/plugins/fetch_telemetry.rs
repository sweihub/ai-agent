// Source: ~/claudecode/openclaudecode/src/utils/plugins/fetchTelemetry.ts
#![allow(dead_code)]

use std::collections::HashSet;

/// Telemetry for plugin/marketplace fetches that hit the network.

pub enum PluginFetchSource {
    InstallCounts,
    MarketplaceClone,
    MarketplacePull,
    MarketplaceUrl,
    PluginClone,
    Mcpb,
}

impl PluginFetchSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InstallCounts => "install_counts",
            Self::MarketplaceClone => "marketplace_clone",
            Self::MarketplacePull => "marketplace_pull",
            Self::MarketplaceUrl => "marketplace_url",
            Self::PluginClone => "plugin_clone",
            Self::Mcpb => "mcpb",
        }
    }
}

pub enum PluginFetchOutcome {
    Success,
    Failure,
    CacheHit,
}

impl PluginFetchOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::CacheHit => "cache_hit",
        }
    }
}

/// Allowlist of public hosts we report by name.
fn known_public_hosts() -> HashSet<&'static str> {
    HashSet::from([
        "github.com",
        "raw.githubusercontent.com",
        "objects.githubusercontent.com",
        "gist.githubusercontent.com",
        "gitlab.com",
        "bitbucket.org",
        "codeberg.org",
        "dev.azure.com",
        "ssh.dev.azure.com",
        "storage.googleapis.com",
    ])
}

/// Extract hostname from a URL and bucket to the allowlist.
fn extract_host(url_or_spec: &str) -> String {
    // Check SCP-style: user@host:path
    if let Some(pos) = url_or_spec.find('@') {
        if let Some(colon_pos) = url_or_spec[pos..].find(':') {
            let host = &url_or_spec[pos + 1..pos + colon_pos];
            let normalized = host.to_lowercase();
            return if known_public_hosts().contains(normalized.as_str()) {
                normalized
            } else {
                "other".to_string()
            };
        }
    }

    // Try parsing as URL
    match url::Url::parse(url_or_spec) {
        Ok(url) => {
            let normalized = url.host_str().unwrap_or("").to_lowercase();
            if known_public_hosts().contains(normalized.as_str()) {
                normalized
            } else {
                "other".to_string()
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

/// Check if URL points at anthropics/claude-plugins-official.
fn is_official_repo(url_or_spec: &str) -> bool {
    url_or_spec.contains("anthropics/claude-plugins-official")
}

/// Log a plugin fetch event for telemetry.
pub fn log_plugin_fetch(
    source: PluginFetchSource,
    url_or_spec: Option<&str>,
    outcome: PluginFetchOutcome,
    duration_ms: u64,
    error_kind: Option<&str>,
) {
    let host = url_or_spec
        .map(extract_host)
        .unwrap_or_else(|| "unknown".to_string());
    let is_official = url_or_spec.map_or(false, is_official_repo);

    let mut metadata = std::collections::HashMap::new();
    metadata.insert("source".to_string(), serde_json::json!(source.as_str()));
    metadata.insert("host".to_string(), serde_json::json!(host));
    metadata.insert("is_official".to_string(), serde_json::json!(is_official));
    metadata.insert("outcome".to_string(), serde_json::json!(outcome.as_str()));
    metadata.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
    metadata.insert("error_kind".to_string(), serde_json::json!(error_kind.unwrap_or("")));

    crate::services::analytics::log_event("tengu_plugin_remote_fetch", metadata);
}

/// Classify an error into a stable bucket for telemetry.
pub fn classify_fetch_error(error: &dyn std::error::Error) -> String {
    let msg = error.to_string().to_lowercase();

    if msg.contains("enotfound")
        || msg.contains("econnrefused")
        || msg.contains("eai_again")
        || msg.contains("could not resolve host")
        || msg.contains("connection refused")
    {
        return "dns_or_refused".to_string();
    }
    if msg.contains("etimedout") || msg.contains("timed out") || msg.contains("timeout") {
        return "timeout".to_string();
    }
    if msg.contains("econnreset")
        || msg.contains("socket hang up")
        || msg.contains("connection reset by peer")
        || msg.contains("remote end hung up")
    {
        return "conn_reset".to_string();
    }
    if msg.contains("403") || msg.contains("401") || msg.contains("permission denied") {
        return "auth".to_string();
    }
    if msg.contains("404") || msg.contains("not found") || msg.contains("repository not found") {
        return "not_found".to_string();
    }
    if msg.contains("certificate") || msg.contains("ssl") || msg.contains("tls") {
        return "tls".to_string();
    }
    if msg.contains("invalid response format") || msg.contains("invalid marketplace schema") {
        return "invalid_schema".to_string();
    }
    "other".to_string()
}
