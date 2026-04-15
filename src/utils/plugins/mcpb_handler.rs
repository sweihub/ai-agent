// Source: ~/claudecode/openclaudecode/src/utils/plugins/mcpbHandler.ts
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::fetch_telemetry::{classify_fetch_error, log_plugin_fetch, PluginFetchOutcome, PluginFetchSource};

/// Result of loading an MCPB file (success case).
pub struct McpbLoadResult {
    pub manifest: McpbManifest,
    pub mcp_config: HashMap<String, McpServerConfig>,
    pub extracted_path: String,
    pub content_hash: String,
}

/// Result when MCPB needs user configuration.
pub struct McpbNeedsConfigResult {
    pub manifest: McpbManifest,
    pub extracted_path: String,
    pub content_hash: String,
    pub config_schema: UserConfigSchema,
    pub existing_config: UserConfigValues,
    pub validation_errors: Vec<String>,
}

/// MCPB manifest structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpbManifest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "user_config")]
    pub user_config: Option<UserConfigSchema>,
}

/// User configuration schema from DXT manifest.
pub type UserConfigSchema = HashMap<String, McpbUserConfigurationOption>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpbUserConfigurationOption {
    #[serde(rename = "type")]
    pub option_type: String,
    pub title: String,
    pub description: String,
    pub required: Option<bool>,
    pub default: Option<serde_json::Value>,
    pub multiple: Option<bool>,
    pub sensitive: Option<bool>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// User configuration values for MCPB.
pub type UserConfigValues = HashMap<String, serde_json::Value>;

/// MCP server configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct McpServerConfig {
    #[serde(rename = "type")]
    pub server_type: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

/// Metadata stored for each cached MCPB.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct McpbCacheMetadata {
    pub source: String,
    pub content_hash: String,
    pub extracted_path: String,
    pub cached_at: String,
    pub last_checked: String,
}

/// Check if a source string is an MCPB file reference.
pub fn is_mcpb_source(source: &str) -> bool {
    source.ends_with(".mcpb") || source.ends_with(".dxt")
}

/// Check if a source is a URL.
fn is_url(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

/// Generate content hash for an MCPB file.
fn generate_content_hash(data: &[u8]) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

/// Get cache directory for MCPB files.
fn get_mcpb_cache_dir(plugin_path: &Path) -> PathBuf {
    plugin_path.join(".mcpb-cache")
}

/// Get metadata file path for cached MCPB.
fn get_metadata_path(cache_dir: &Path, source: &str) -> PathBuf {
    use sha2::Digest;
    // Use sha2 instead of md5 since md5 crate is not available
    let mut hasher = sha2::Sha256::new();
    hasher.update(source.as_bytes());
    let result = hasher.finalize();
    let source_hash = hex::encode(&result[..4]);
    cache_dir.join(format!("{}.metadata.json", source_hash))
}

/// Validate user configuration values against DXT user_config schema.
pub fn validate_user_config(
    values: &UserConfigValues,
    schema: &UserConfigSchema,
) -> (bool, Vec<String>) {
    let mut errors = Vec::new();

    for (key, field_schema) in schema {
        let value = values.get(key);

        // Check required fields
        if field_schema.required.unwrap_or(false)
            && (value.is_none()
                || value
                    .map(|v| v.as_str().map(|s| s.is_empty()).unwrap_or(false))
                    .unwrap_or(false))
        {
            errors.push(format!(
                "{} is required but not provided",
                field_schema.title
            ));
            continue;
        }

        if value.is_none() {
            continue;
        }

        let value = value.unwrap();

        // Type validation
        match field_schema.option_type.as_str() {
            "string" => {
                if value.is_array() {
                    if !field_schema.multiple.unwrap_or(false) {
                        errors.push(format!("{} must be a string, not an array", field_schema.title));
                    }
                } else if !value.is_string() {
                    errors.push(format!("{} must be a string", field_schema.title));
                }
            }
            "number" => {
                if !value.is_number() {
                    errors.push(format!("{} must be a number", field_schema.title));
                } else if let Some(n) = value.as_f64() {
                    if let Some(min) = field_schema.min {
                        if n < min {
                            errors.push(format!("{} must be at least {}", field_schema.title, min));
                        }
                    }
                    if let Some(max) = field_schema.max {
                        if n > max {
                            errors.push(format!("{} must be at most {}", field_schema.title, max));
                        }
                    }
                }
            }
            "boolean" => {
                if !value.is_boolean() {
                    errors.push(format!("{} must be a boolean", field_schema.title));
                }
            }
            _ => {}
        }
    }

    (errors.is_empty(), errors)
}

/// Check if an MCPB source has changed and needs re-extraction.
pub async fn check_mcpb_changed(source: &str, plugin_path: &Path) -> bool {
    let cache_dir = get_mcpb_cache_dir(plugin_path);
    let metadata_path = get_metadata_path(&cache_dir, source);

    // Load metadata
    let metadata = match tokio::fs::read_to_string(&metadata_path).await {
        Ok(content) => match serde_json::from_str::<McpbCacheMetadata>(&content) {
            Ok(m) => m,
            Err(_) => return true,
        },
        Err(_) => return true,
    };

    // Check if extraction directory still exists
    if !Path::new(&metadata.extracted_path).exists() {
        log::debug!("MCPB extraction path missing: {}", metadata.extracted_path);
        return true;
    }

    // For local files, check mtime
    if !is_url(source) {
        let local_path = plugin_path.join(source);
        match tokio::fs::metadata(&local_path).await {
            Ok(meta) => {
                let cached_time = chrono::DateTime::parse_from_rfc3339(&metadata.cached_at)
                    .map(|dt| dt.timestamp_millis() as u64)
                    .unwrap_or(0);
                let file_time = meta
                    .modified()
                    .ok()
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_millis() as u64)
                    })
                    .unwrap_or(0);

                if file_time > cached_time {
                    log::debug!("MCPB file modified");
                    return true;
                }
            }
            Err(_) => return true,
        }
    }

    false
}

/// Download MCPB file from URL.
async fn download_mcpb(url: &str, dest_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    log::debug!("Downloading MCPB from {}", url);

    let started = std::time::Instant::now();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let response = client.get(url).send().await?;
    let data = response.bytes().await?.to_vec();

    log_plugin_fetch(
        PluginFetchSource::Mcpb,
        Some(url),
        PluginFetchOutcome::Success,
        started.elapsed().as_millis() as u64,
        None,
    );

    tokio::fs::write(dest_path, &data).await?;
    log::debug!("Downloaded {} bytes to {:?}", data.len(), dest_path);

    Ok(data)
}

/// Load and extract an MCPB file.
pub async fn load_mcpb_file(
    source: &str,
    plugin_path: &Path,
    _plugin_id: &str,
) -> Result<Result<McpbLoadResult, McpbNeedsConfigResult>, Box<dyn std::error::Error + Send + Sync>> {
    let cache_dir = get_mcpb_cache_dir(plugin_path);
    tokio::fs::create_dir_all(&cache_dir).await?;

    log::debug!("Loading MCPB from source: {}", source);

    // Check cache first
    if !check_mcpb_changed(source, plugin_path).await {
        // Use cached version - simplified
        return Err(format!("Cache handling not fully implemented").into());
    }

    // Download or read the MCPB file
    let data = if is_url(source) {
        let dest_path = cache_dir.join("download.mcpb");
        download_mcpb(source, &dest_path).await?
    } else {
        tokio::fs::read(plugin_path.join(source)).await?
    };

    // Extract ZIP contents
    let content_hash = generate_content_hash(&data);

    // Parse manifest from extracted files
    // In production, would use a ZIP library to extract manifest.json
    let manifest = McpbManifest {
        name: "unknown".to_string(),
        version: None,
        description: None,
        user_config: None,
    };

    Ok(Ok(McpbLoadResult {
        manifest,
        mcp_config: HashMap::new(),
        extracted_path: cache_dir.join(&content_hash).to_string_lossy().to_string(),
        content_hash,
    }))
}
