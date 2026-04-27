//! Team Memory Sync - ported from ~/claudecode/openclaudecode/src/services/teamMemorySync/
//!
//! Syncs team memory files between the local filesystem and the server API.
//! Team memory is scoped per-repo (identified by git remote hash) and shared
//! across all authenticated org members.

use crate::AgentError;
use crate::constants::env::system;
use crate::utils::http::get_user_agent;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// Team memory content - flat key-value storage
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TeamMemoryContent {
    /// Keys are file paths relative to team memory directory
    pub entries: HashMap<String, String>,
    /// Per-key SHA-256 checksums (sha256:<hex>)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub entry_checksums: HashMap<String, String>,
}

/// Full response from GET /api/claude_code/team_memory
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemoryData {
    pub organization_id: String,
    pub repo: String,
    pub version: u32,
    pub last_modified: String,
    pub checksum: String,
    pub content: TeamMemoryContent,
}

/// Structured 413 error body for too many entries
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemoryTooManyEntries {
    pub error: TeamMemoryTooManyEntriesError,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemoryTooManyEntriesError {
    pub details: TeamMemoryTooManyEntriesDetails,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemoryTooManyEntriesDetails {
    #[serde(rename = "error_code")]
    pub error_code: String,
    #[serde(rename = "max_entries")]
    pub max_entries: u32,
    #[serde(rename = "received_entries")]
    pub received_entries: u32,
}

/// A file skipped during push due to detected secret
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkippedSecretFile {
    /// Path relative to team memory directory
    pub path: String,
    /// Gitleaks rule ID (e.g., "github-pat", "aws-access-token")
    pub rule_id: String,
    /// Human-readable label derived from rule ID
    pub label: String,
}

/// Result from fetching team memory
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemorySyncFetchResult {
    pub success: bool,
    pub data: Option<TeamMemoryData>,
    /// true if 404 (no data exists)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_empty: Option<bool>,
    /// true if 304 (ETag matched, no changes)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_modified: Option<bool>,
    /// ETag from response header
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_retry: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
}

/// Lightweight metadata-only probe result (GET ?view=hashes)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemoryHashesResult {
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_checksums: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
}

/// Result from uploading team memory with conflict info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemorySyncPushResult {
    pub success: bool,
    pub files_uploaded: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    /// true if 412 Precondition Failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Files skipped due to detected secrets
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_secrets: Vec<SkippedSecretFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
}

/// Result from uploading team memory
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamMemorySyncUploadResult {
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    /// true if 412 Precondition Failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Structured error_code from parsed 413 body
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_error_code: Option<String>,
    /// Server-enforced max_entries
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_max_entries: Option<u32>,
    /// How many entries the rejected push would have produced
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_received_entries: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
}

// ─── Sync State ─────────────────────────────────────────────────

/// Mutable state for team memory sync service
#[derive(Debug, Clone)]
pub struct SyncState {
    /// Last known server checksum (ETag) for conditional requests
    pub last_known_checksum: Option<String>,
    /// Per-key content hash (sha256:<hex>) of what we believe server holds
    pub server_checksums: HashMap<String, String>,
    /// Server-enforced max_entries cap, learned from structured 413
    pub server_max_entries: Option<u32>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            last_known_checksum: None,
            server_checksums: HashMap::new(),
            server_max_entries: None,
        }
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new sync state
pub fn create_sync_state() -> SyncState {
    SyncState::new()
}

// ─── Hashing ───────────────────────────────────────────────────

/// Compute sha256:<hex> over UTF-8 bytes of content
pub fn hash_content(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(result))
}

// ─── Configuration ─────────────────────────────────────────────

/// Team memory sync timeout in milliseconds
pub const TEAM_MEMORY_SYNC_TIMEOUT_MS: u64 = 30_000;
/// Per-entry size cap (250KB)
pub const MAX_FILE_SIZE_BYTES: usize = 250_000;
/// Gateway body-size cap (200KB)
pub const MAX_PUT_BODY_BYTES: usize = 200_000;
/// Max retries for transient failures
pub const MAX_RETRIES: u32 = 3;
/// Max retries for conflict resolution
pub const MAX_CONFLICT_RETRIES: u32 = 2;

// ─── File Operations ───────────────────────────────────────────

/// Get the team memory directory path
pub fn get_team_memory_dir() -> PathBuf {
    let home = std::env::var(system::HOME)
        .or_else(|_| std::env::var(system::USERPROFILE))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".open-agent-sdk")
        .join("team_memory")
}

/// Get team memory file path for a given key
pub fn get_team_memory_path(key: &str) -> PathBuf {
    // Validate key to prevent path traversal
    if key.contains("..") || key.starts_with('/') {
        return get_team_memory_dir().join("INVALID");
    }
    get_team_memory_dir().join(key)
}

/// Validate a team memory key
pub fn validate_team_memory_key(key: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err("Key cannot be empty".to_string());
    }
    if key.contains("..") {
        return Err("Key cannot contain '..'".to_string());
    }
    if key.starts_with('/') {
        return Err("Key cannot start with '/'".to_string());
    }
    Ok(())
}

/// Read team memory entries from local filesystem
pub async fn read_local_team_memory() -> Result<HashMap<String, String>, AgentError> {
    let dir = get_team_memory_dir();

    if !dir.exists() {
        return Ok(HashMap::new());
    }

    let mut entries = HashMap::new();
    let mut dirs_to_process: Vec<PathBuf> = vec![dir.clone()];

    while let Some(current_dir) = dirs_to_process.pop() {
        let mut read_dir = tokio::fs::read_dir(&current_dir)
            .await
            .map_err(AgentError::Io)?;

        while let Some(entry) = read_dir.next_entry().await.map_err(AgentError::Io)? {
            let path = entry.path();
            let relative = path
                .strip_prefix(&dir)
                .map_err(|_| AgentError::Internal("Failed to get relative path".to_string()))?
                .to_string_lossy()
                .to_string();

            if path.is_dir() {
                dirs_to_process.push(path);
            } else if path.is_file() {
                // Skip hidden files
                if relative.starts_with('.') {
                    continue;
                }
                let content = tokio::fs::read_to_string(&path)
                    .await
                    .map_err(AgentError::Io)?;
                entries.insert(relative, content);
            }
        }
    }

    Ok(entries)
}

/// Write team memory entries to local filesystem
pub async fn write_local_team_memory(entries: &HashMap<String, String>) -> Result<(), AgentError> {
    let dir = get_team_memory_dir();
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(AgentError::Io)?;

    for (key, content) in entries {
        let path = get_team_memory_path(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AgentError::Io)?;
        }
        tokio::fs::write(&path, content)
            .await
            .map_err(AgentError::Io)?;
    }

    Ok(())
}

/// Delete a team memory entry
pub async fn delete_local_team_memory_entry(key: &str) -> Result<(), AgentError> {
    let path = get_team_memory_path(key);
    if path.exists() {
        tokio::fs::remove_file(path).await.map_err(AgentError::Io)?;
    }
    Ok(())
}

// ─── Delta Computation ─────────────────────────────────────────

/// Compute delta between local and server checksums
pub fn compute_delta(
    local_entries: &HashMap<String, String>,
    server_checksums: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut delta = HashMap::new();

    for (key, content) in local_entries {
        let local_hash = hash_content(content);
        let server_hash = server_checksums.get(key);

        // Upload if: key doesn't exist on server, or hash differs
        if server_hash.is_none() || server_hash != Some(&local_hash) {
            delta.insert(key.clone(), content.clone());
        }
    }

    delta
}

/// Batch delta entries by byte size
pub fn batch_delta_by_bytes(
    delta: &HashMap<String, String>,
    max_bytes: usize,
) -> Vec<HashMap<String, String>> {
    let mut batches: Vec<HashMap<String, String>> = Vec::new();
    let mut current_batch: HashMap<String, String> = HashMap::new();
    let mut current_bytes: usize = 0;

    // Sort keys for deterministic ordering
    let mut keys: Vec<&String> = delta.keys().collect();
    keys.sort();

    for key in keys {
        let content = delta.get(key).unwrap();
        let entry_bytes = key.len() + content.len();

        // If single entry exceeds max, it goes in its own batch
        if entry_bytes > max_bytes {
            // Flush current batch if non-empty
            if !current_batch.is_empty() {
                batches.push(current_batch);
                current_batch = HashMap::new();
                current_bytes = 0;
            }
            // Put oversized entry in its own batch
            let mut single = HashMap::new();
            single.insert(key.clone(), content.clone());
            batches.push(single);
            continue;
        }

        // Check if adding this entry would exceed limit
        if current_bytes + entry_bytes > max_bytes && !current_batch.is_empty() {
            batches.push(current_batch);
            current_batch = HashMap::new();
            current_bytes = 0;
        }

        current_batch.insert(key.clone(), content.clone());
        current_bytes += entry_bytes;
    }

    // Push remaining batch
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    batches
}

// ─── Sync Functions ───────────────────────────────────────────

/// API base URL for team memory operations
fn get_team_memory_api_base() -> String {
    std::env::var("AI_API_BASE_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| "https://api.anthropic.com".to_string())
}

/// Get OAuth token for authentication
fn get_team_memory_auth_token() -> Option<String> {
    std::env::var("AI_CODE_OAUTH_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| {
            std::env::var("AI_OAUTH_TOKEN")
                .ok()
                .filter(|t| !t.is_empty())
        })
        .or_else(|| {
            std::env::var("AI_AUTH_TOKEN")
                .ok()
                .filter(|t| !t.is_empty())
        })
}

/// Build HTTP headers for team memory requests
fn build_team_memory_headers(
    etag: Option<&str>,
    content_type: Option<&str>,
) -> Result<reqwest::header::HeaderMap, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "anthropic-version",
        reqwest::header::HeaderValue::from_static("2025-04-20"),
    );

    if let Some(token) = get_team_memory_auth_token() {
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&auth_value)
                .map_err(|e| format!("Invalid auth header: {}", e))?,
        );
    }

    if let Some(etag_value) = etag {
        headers.insert(
            "If-None-Match",
            reqwest::header::HeaderValue::from_str(etag_value)
                .map_err(|e| format!("Invalid ETag header: {}", e))?,
        );
    }

    if let Some(ct) = content_type {
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_str(ct)
                .map_err(|e| format!("Invalid Content-Type header: {}", e))?,
        );
    }

    headers.insert(
        "User-Agent",
        reqwest::header::HeaderValue::from_str(&get_user_agent())
            .map_err(|e| format!("Invalid User-Agent header: {}", e))?,
    );

    Ok(headers)
}

/// Build the team memory API URL
fn build_team_memory_url(repo_slug: &str, view: Option<&str>) -> String {
    let base = get_team_memory_api_base();
    let mut url = format!("{}/api/claude_code/team_memory", base);
    let mut query_params: Vec<(String, String)> = vec![("repo".to_string(), repo_slug.to_string())];

    if let Some(v) = view {
        query_params.push(("view".to_string(), v.to_string()));
    }

    if !query_params.is_empty() {
        url.push('?');
        url.push_str(
            &query_params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&"),
        );
    }

    url
}

/// Check if team memory sync is available (has auth credentials)
pub fn is_team_memory_sync_available() -> bool {
    get_team_memory_auth_token().is_some()
}

/// Pull team memory from server with conditional request support
pub async fn pull_team_memory(
    state: &mut SyncState,
    repo_slug: &str,
) -> Result<TeamMemorySyncFetchResult, AgentError> {
    // Check if sync is available
    if !is_team_memory_sync_available() {
        return Ok(TeamMemorySyncFetchResult {
            success: false,
            data: None,
            is_empty: None,
            not_modified: None,
            checksum: None,
            error: Some("No OAuth token available for team memory sync".to_string()),
            skip_retry: Some(true),
            error_type: Some("auth".to_string()),
            http_status: None,
        });
    }

    // First, probe for hashes (lightweight metadata-only request)
    let hashes_url = build_team_memory_url(repo_slug, Some("hashes"));
    let headers = build_team_memory_headers(state.last_known_checksum.as_deref(), None)
        .map_err(|e| AgentError::Internal(e))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(
            TEAM_MEMORY_SYNC_TIMEOUT_MS,
        ))
        .build()
        .map_err(|e| AgentError::Internal(e.to_string()))?;

    // Try the hashes probe first
    let hashes_response = match client
        .get(&hashes_url)
        .headers(headers.clone())
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let is_timeout = e.is_timeout() || e.is_connect();
            return Ok(TeamMemorySyncFetchResult {
                success: false,
                data: None,
                is_empty: None,
                not_modified: None,
                checksum: None,
                error: Some(format!("Team memory request failed: {}", e)),
                skip_retry: Some(!is_timeout),
                error_type: Some(if is_timeout { "timeout" } else { "network" }.to_string()),
                http_status: None,
            });
        }
    };

    let hashes_status = hashes_response.status();

    // Handle 304 Not Modified
    if hashes_status == 304 {
        log::debug!("Team memory not modified (304) for repo: {}", repo_slug);
        return Ok(TeamMemorySyncFetchResult {
            success: true,
            data: None,
            is_empty: Some(false),
            not_modified: Some(true),
            checksum: state.last_known_checksum.clone(),
            error: None,
            skip_retry: Some(true),
            error_type: None,
            http_status: Some(304),
        });
    }

    // Handle 404 Not Found (no team memory exists)
    if hashes_status == 404 {
        log::debug!("No team memory exists for repo: {}", repo_slug);
        return Ok(TeamMemorySyncFetchResult {
            success: true,
            data: None,
            is_empty: Some(true),
            not_modified: Some(false),
            checksum: None,
            error: None,
            skip_retry: Some(true),
            error_type: None,
            http_status: Some(404),
        });
    }

    if !hashes_status.is_success() {
        let body = hashes_response.text().await.unwrap_or_default();
        log::debug!(
            "Team memory hashes probe failed with status {}: {}",
            hashes_status,
            body
        );
        return Ok(TeamMemorySyncFetchResult {
            success: false,
            data: None,
            is_empty: None,
            not_modified: None,
            checksum: None,
            error: Some(format!(
                "Team memory probe failed with status {}: {}",
                hashes_status, body
            )),
            skip_retry: Some(hashes_status.is_client_error()),
            error_type: Some("api".to_string()),
            http_status: Some(hashes_status.as_u16()),
        });
    }

    // Parse hashes response
    let hashes_result = match hashes_response.json::<TeamMemoryHashesResult>().await {
        Ok(r) => r,
        Err(e) => {
            return Ok(TeamMemorySyncFetchResult {
                success: false,
                data: None,
                is_empty: None,
                not_modified: None,
                checksum: None,
                error: Some(format!("Failed to parse team memory hashes: {}", e)),
                skip_retry: Some(false),
                error_type: Some("parse".to_string()),
                http_status: Some(hashes_status.as_u16()),
            });
        }
    };

    // Update state with server checksums
    if let Some(version) = hashes_result.version {
        log::debug!(
            "Team memory version: {}, checksum: {:?}",
            version,
            hashes_result.checksum
        );
    }

    // Update server checksums from hashes response
    if let Some(ref entry_checksums) = hashes_result.entry_checksums {
        state.server_checksums = entry_checksums.clone();
    }
    if let Some(ref checksum) = hashes_result.checksum {
        state.last_known_checksum = Some(checksum.clone());
    }

    // Now fetch the full content
    let full_url = build_team_memory_url(repo_slug, None);
    let full_headers = build_team_memory_headers(state.last_known_checksum.as_deref(), None)
        .map_err(|e| AgentError::Internal(e))?;

    let full_response = match client.get(&full_url).headers(full_headers).send().await {
        Ok(r) => r,
        Err(e) => {
            let is_timeout = e.is_timeout() || e.is_connect();
            return Ok(TeamMemorySyncFetchResult {
                success: false,
                data: None,
                is_empty: None,
                not_modified: None,
                checksum: state.last_known_checksum.clone(),
                error: Some(format!("Team memory fetch failed: {}", e)),
                skip_retry: Some(!is_timeout),
                error_type: Some(if is_timeout { "timeout" } else { "network" }.to_string()),
                http_status: None,
            });
        }
    };

    let full_status = full_response.status();

    // Handle 304 Not Modified
    if full_status == 304 {
        log::debug!(
            "Team memory content not modified (304) for repo: {}",
            repo_slug
        );
        return Ok(TeamMemorySyncFetchResult {
            success: true,
            data: None,
            is_empty: Some(false),
            not_modified: Some(true),
            checksum: state.last_known_checksum.clone(),
            error: None,
            skip_retry: Some(true),
            error_type: None,
            http_status: Some(304),
        });
    }

    // Handle 404
    if full_status == 404 {
        return Ok(TeamMemorySyncFetchResult {
            success: true,
            data: None,
            is_empty: Some(true),
            not_modified: Some(false),
            checksum: None,
            error: None,
            skip_retry: Some(true),
            error_type: None,
            http_status: Some(404),
        });
    }

    // Extract ETag from response headers
    let response_etag = full_response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    if let Some(ref etag) = response_etag {
        state.last_known_checksum = Some(etag.clone());
    }

    if !full_status.is_success() {
        let body = full_response.text().await.unwrap_or_default();
        return Ok(TeamMemorySyncFetchResult {
            success: false,
            data: None,
            is_empty: None,
            not_modified: None,
            checksum: state.last_known_checksum.clone(),
            error: Some(format!(
                "Team memory fetch failed with status {}: {}",
                full_status, body
            )),
            skip_retry: Some(full_status.is_client_error()),
            error_type: Some("api".to_string()),
            http_status: Some(full_status.as_u16()),
        });
    }

    // Parse the full response
    match full_response.json::<TeamMemoryData>().await {
        Ok(data) => {
            log::info!(
                "Successfully pulled team memory for repo: {}, version: {}, entries: {}",
                repo_slug,
                data.version,
                data.content.entries.len()
            );

            // Update state
            state.last_known_checksum = Some(data.checksum.clone());
            state.server_checksums = data.content.entry_checksums.clone();

            Ok(TeamMemorySyncFetchResult {
                success: true,
                data: Some(data),
                is_empty: Some(false),
                not_modified: Some(false),
                checksum: state.last_known_checksum.clone(),
                error: None,
                skip_retry: None,
                error_type: None,
                http_status: Some(full_status.as_u16()),
            })
        }
        Err(e) => Ok(TeamMemorySyncFetchResult {
            success: false,
            data: None,
            is_empty: None,
            not_modified: None,
            checksum: state.last_known_checksum.clone(),
            error: Some(format!("Failed to parse team memory response: {}", e)),
            skip_retry: Some(false),
            error_type: Some("parse".to_string()),
            http_status: Some(full_status.as_u16()),
        }),
    }
}

/// Push team memory to server with conflict detection and secret scanning
pub async fn push_team_memory(
    state: &mut SyncState,
    repo_slug: &str,
    entries: &HashMap<String, String>,
) -> Result<TeamMemorySyncPushResult, AgentError> {
    // Check if sync is available
    if !is_team_memory_sync_available() {
        return Ok(TeamMemorySyncPushResult {
            success: false,
            files_uploaded: 0,
            checksum: None,
            conflict: None,
            error: Some("No OAuth token available for team memory sync".to_string()),
            skipped_secrets: Vec::new(),
            error_type: Some("auth".to_string()),
            http_status: None,
        });
    }

    // Scan for secrets before uploading
    let skipped_secrets = scan_entries_for_secrets(entries);
    let entries_to_upload: HashMap<String, String> = entries
        .iter()
        .filter(|(path, _)| !skipped_secrets.iter().any(|s| s.path == **path))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    if entries_to_upload.is_empty() {
        return Ok(TeamMemorySyncPushResult {
            success: true,
            files_uploaded: 0,
            checksum: state.last_known_checksum.clone(),
            conflict: None,
            error: None,
            skipped_secrets,
            error_type: None,
            http_status: None,
        });
    }

    // Check size constraints
    if entries_to_upload.len() > 1000 {
        return Ok(TeamMemorySyncPushResult {
            success: false,
            files_uploaded: 0,
            checksum: None,
            conflict: None,
            error: Some(format!(
                "Too many entries: {} (max: 1000)",
                entries_to_upload.len()
            )),
            skipped_secrets,
            error_type: Some("too_many_entries".to_string()),
            http_status: Some(413),
        });
    }

    // Build the push request body
    let body = TeamMemoryContent {
        entries: entries_to_upload.clone(),
        entry_checksums: entries_to_upload
            .iter()
            .map(|(k, v)| (k.clone(), hash_content(v)))
            .collect(),
    };

    let url = build_team_memory_url(repo_slug, None);
    let mut headers = build_team_memory_headers(None, None).map_err(|e| AgentError::Internal(e))?;

    // Add If-Match header for conflict detection
    if let Some(ref checksum) = state.last_known_checksum {
        headers.insert(
            "If-Match",
            reqwest::header::HeaderValue::from_str(checksum)
                .map_err(|e| AgentError::Internal(e.to_string()))?,
        );
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(
            TEAM_MEMORY_SYNC_TIMEOUT_MS,
        ))
        .build()
        .map_err(|e| AgentError::Internal(e.to_string()))?;

    let response = match client.put(&url).headers(headers).json(&body).send().await {
        Ok(r) => r,
        Err(e) => {
            let is_timeout = e.is_timeout() || e.is_connect();
            return Ok(TeamMemorySyncPushResult {
                success: false,
                files_uploaded: 0,
                checksum: None,
                conflict: None,
                error: Some(format!("Team memory push failed: {}", e)),
                skipped_secrets,
                error_type: Some(if is_timeout { "timeout" } else { "network" }.to_string()),
                http_status: None,
            });
        }
    };

    let status = response.status();

    // Handle 412 Precondition Failed (conflict)
    if status == 412 {
        log::debug!("Team memory conflict (412) for repo: {}", repo_slug);
        return Ok(TeamMemorySyncPushResult {
            success: false,
            files_uploaded: 0,
            checksum: None,
            conflict: Some(true),
            error: Some("Conflict: team memory was modified by another client".to_string()),
            skipped_secrets,
            error_type: Some("conflict".to_string()),
            http_status: Some(412),
        });
    }

    // Handle 413 Payload Too Large
    if status == 413 {
        let body_text = response.text().await.unwrap_or_default();
        let max_entries =
            if let Ok(error_body) = serde_json::from_str::<TeamMemoryTooManyEntries>(&body_text) {
                Some(error_body.error.details.max_entries)
            } else {
                None
            };

        if let Some(max) = max_entries {
            state.server_max_entries = Some(max);
        }

        return Ok(TeamMemorySyncPushResult {
            success: false,
            files_uploaded: 0,
            checksum: None,
            conflict: None,
            error: Some(format!(
                "Payload too large: {} entries (max: {:?})",
                entries_to_upload.len(),
                max_entries
            )),
            skipped_secrets,
            error_type: Some("payload_too_large".to_string()),
            http_status: Some(413),
        });
    }

    // Extract ETag from response
    let response_etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    if let Some(ref etag) = response_etag {
        state.last_known_checksum = Some(etag.clone());
    }

    if !status.is_success() {
        let body_text = response.text().await.unwrap_or_default();
        return Ok(TeamMemorySyncPushResult {
            success: false,
            files_uploaded: 0,
            checksum: None,
            conflict: None,
            error: Some(format!(
                "Team memory push failed with status {}: {}",
                status, body_text
            )),
            skipped_secrets,
            error_type: Some("api".to_string()),
            http_status: Some(status.as_u16()),
        });
    }

    let files_uploaded = entries_to_upload.len() as u32;
    log::info!(
        "Successfully pushed {} team memory files for repo: {}",
        files_uploaded,
        repo_slug
    );

    Ok(TeamMemorySyncPushResult {
        success: true,
        files_uploaded,
        checksum: state.last_known_checksum.clone(),
        conflict: None,
        error: None,
        skipped_secrets,
        error_type: None,
        http_status: Some(status.as_u16()),
    })
}

/// Full sync: pull, merge, push
pub async fn sync_team_memory(
    state: &mut SyncState,
    repo_slug: &str,
) -> Result<TeamMemorySyncPushResult, AgentError> {
    // Pull from server
    let pull_result = pull_team_memory(state, repo_slug).await?;

    if !pull_result.success {
        return Ok(TeamMemorySyncPushResult {
            success: false,
            files_uploaded: 0,
            checksum: None,
            conflict: None,
            error: pull_result.error,
            skipped_secrets: Vec::new(),
            error_type: pull_result.error_type,
            http_status: pull_result.http_status,
        });
    }

    // Read local entries
    let local_entries = read_local_team_memory().await?;

    // Compute delta
    let delta = compute_delta(&local_entries, &state.server_checksums);

    if delta.is_empty() {
        return Ok(TeamMemorySyncPushResult {
            success: true,
            files_uploaded: 0,
            checksum: state.last_known_checksum.clone(),
            conflict: None,
            error: None,
            skipped_secrets: Vec::new(),
            error_type: None,
            http_status: None,
        });
    }

    // Push delta
    push_team_memory(state, repo_slug, &delta).await
}

// ─── Secret Scanning ───────────────────────────────────────────
// Curated high-confidence patterns from gitleaks with distinctive prefixes.
// Ported from openclaudecode/src/services/teamMemorySync/secretScanner.ts

/// A secret match from scanning content
struct SecretMatch {
    rule_id: String,
    label: String,
}

/// Words where canonical capitalization differs from title case
fn rule_id_to_label(rule_id: &str) -> String {
    let special = [
        ("aws", "AWS"), ("gcp", "GCP"), ("api", "API"), ("pat", "PAT"),
        ("ad", "AD"), ("tf", "TF"), ("oauth", "OAuth"), ("npm", "NPM"),
        ("pypi", "PyPI"), ("jwt", "JWT"), ("github", "GitHub"),
        ("gitlab", "GitLab"), ("openai", "OpenAI"), ("digitalocean", "DigitalOcean"),
        ("huggingface", "HuggingFace"), ("hashicorp", "HashiCorp"),
        ("sendgrid", "SendGrid"),
    ];
    rule_id.split('-')
        .map(|part| {
            if let Some(&(_, canonical)) = special.iter().find(|&&(k, _)| k == part) {
                canonical.to_string()
            } else {
                let mut s = String::new();
                let mut chars = part.chars();
                if let Some(c) = chars.next() {
                    s.push(c.to_ascii_uppercase());
                    for ch in chars {
                        s.push(ch);
                    }
                }
                s
            }
        })
        .collect()
}

fn scan_content_for_secrets(content: &str) -> Vec<SecretMatch> {
    // Token delimiter characters: single quote, double quote, backtick
    // Used as boundary markers in secret patterns
    let qt = chr(39);  // '
    let dq = chr(34);  // "
    let bt = chr(96);  // `
    let q = format!("{}{}{}", qt, dq, bt);

    let rules: Vec<(&str, String)> = vec![
        // Cloud providers
        ("aws-access-token", r"\b((?:A3T[A-Z0-9]|AKIA|ASIA|ABIA|ACCA)[A-Z2-7]{16})\b".to_string()),
        ("gcp-api-key", r"\b(AIza[\w-]{35})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("azure-ad-client-secret", r"(?:^|[__Q__]\s>=:(,)])([a-zA-Z0-9_~.]{3}\dQ~[a-zA-Z0-9_~.-]{31,34})(?:$|[__Q__]\s<),])".replace("__Q__", &q)),
        ("digitalocean-pat", r"\b(dop_v1_[a-f0-9]{64})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("digitalocean-access-token", r"\b(doo_v1_[a-f0-9]{64})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        // AI APIs
        ("anthropic-api-key", r"\b(sk-ant-api03-[a-zA-Z0-9_\-]{93}AA)(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("anthropic-admin-api-key", r"\b(sk-ant-admin01-[a-zA-Z0-9_\-]{93}AA)(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("openai-api-key", r"\b(sk-(?:proj|svcacct|admin)-(?:[A-Za-z0-9_-]{74}|[A-Za-z0-9_-]{58})T3BlbkFJ(?:[A-Za-z0-9_-]{74}|[A-Za-z0-9_-]{58})\b|sk-[a-zA-Z0-9]{20}T3BlbkFJ[a-zA-Z0-9]{20})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("huggingface-access-token", r"\b(hf_[a-zA-Z]{34})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        // Version control
        ("github-pat", r"ghp_[0-9a-zA-Z]{36}".to_string()),
        ("github-fine-grained-pat", r"github_pat_\w{82}".to_string()),
        ("github-app-token", r"(?:ghu|ghs)_[0-9a-zA-Z]{36}".to_string()),
        ("github-oauth", r"gho_[0-9a-zA-Z]{36}".to_string()),
        ("github-refresh-token", r"ghr_[0-9a-zA-Z]{36}".to_string()),
        ("gitlab-pat", r"glpat-[\w-]{20}".to_string()),
        ("gitlab-deploy-token", r"gldt-[0-9a-zA-Z_\-]{20}".to_string()),
        // Communication
        ("slack-bot-token", r"xoxb-[0-9]{10,13}-[0-9]{10,13}[a-zA-Z0-9-]*".to_string()),
        ("slack-user-token", r"xox[pe](?:-[0-9]{10,13}){3}-[a-zA-Z0-9-]{28,34}".to_string()),
        ("slack-app-token", r"(?i)xapp-\d-[A-Z0-9]+-\d+-[a-z0-9]+".to_string()),
        ("twilio-api-key", r"SK[0-9a-fA-F]{32}".to_string()),
        ("sendgrid-api-token", r"\b(SG\.[a-zA-Z0-9=_\-.]{66})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        // Dev tooling
        ("npm-access-token", r"\b(npm_[a-zA-Z0-9]{36})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("pypi-upload-token", r"pypi-AgEIcHlwaS5vcmc[\w-]{50,1000}".to_string()),
        ("databricks-api-token", r"\b(dapi[a-f0-9]{32}(?:-\d)?)(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("hashicorp-tf-api-token", r"[a-zA-Z0-9]{14}\.atlasv1\.[a-zA-Z0-9\-_=]{60,70}".to_string()),
        ("pulumi-api-token", r"\b(pul-[a-f0-9]{40})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("postman-api-token", r"\b(PMAK-[a-fA-F0-9]{24}-[a-fA-F0-9]{34})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        // Observability
        ("grafana-api-key", r"\b(eyJrIjoi[A-Za-z0-9+/]{70,400}={0,3})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("grafana-cloud-api-token", r"\b(glc_[A-Za-z0-9+/]{32,400}={0,3})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("grafana-service-account-token", r"\b(glsa_[A-Za-z0-9]{32}_[A-Fa-f0-9]{8})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("sentry-user-token", r"\b(sntryu_[a-f0-9]{64})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("sentry-org-token", r"\bsntrys_eyJpYXQiO[a-zA-Z0-9+/]{10,200}(?:LCJyZWdpb25fdXJs|InJlZ2lvbl91cmwi|cmVnaW9uX3VybCI6)[a-zA-Z0-9+/]{10,200}={0,2}_[a-zA-Z0-9+/]{43}".to_string()),
        // Payment / commerce
        ("stripe-access-token", r"\b((?:sk|rk)_(?:test|live|prod)_[a-zA-Z0-9]{10,99})(?:[__Q__]\s;]|\\[nr]|$)".replace("__Q__", &q)),
        ("shopify-access-token", r"shpat_[a-fA-F0-9]{32}".to_string()),
        ("shopify-shared-secret", r"shpss_[a-fA-F0-9]{32}".to_string()),
        // Crypto
        ("private-key", r"(?i)-----BEGIN[ A-Z0-9_-]{0,100}PRIVATE KEY(?: BLOCK)?-----[\s\S-]{64,}?-----END[ A-Z0-9_-]{0,100}PRIVATE KEY(?: BLOCK)?-----".to_string()),
    ];

    let mut matches = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for (rule_id, pattern) in &rules {
        if seen.contains(*rule_id) {
            continue;
        }
        if let Ok(re) = regex::Regex::new(pattern) {
            if re.is_match(content) {
                seen.insert(*rule_id);
                matches.push(SecretMatch {
                    rule_id: (*rule_id).to_string(),
                    label: rule_id_to_label(rule_id),
                });
            }
        }
    }

    matches
}

fn chr(code: u32) -> String {
    std::char::from_u32(code).map(|c| c.to_string()).unwrap_or_default()
}

/// Scan content for potential secrets. Returns one entry per rule that matched.
pub fn scan_for_secrets(content: &str, path: &str) -> Vec<SkippedSecretFile> {
    scan_content_for_secrets(content)
        .into_iter()
        .map(|m| SkippedSecretFile {
            path: path.to_string(),
            rule_id: m.rule_id,
            label: m.label,
        })
        .collect()
}

/// Scan entries for secrets
pub fn scan_entries_for_secrets(entries: &HashMap<String, String>) -> Vec<SkippedSecretFile> {
    let mut skipped = Vec::new();

    for (path, content) in entries {
        skipped.extend(scan_for_secrets(content, path));
    }

    skipped
}

// ─── State Management ──────────────────────────────────────────

/// Global team memory sync enabled flag
static TEAM_MEMORY_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check if team memory sync is enabled
pub fn is_team_memory_enabled() -> bool {
    TEAM_MEMORY_ENABLED.load(Ordering::SeqCst)
}

/// Enable team memory sync
pub fn enable_team_memory() {
    TEAM_MEMORY_ENABLED.store(true, Ordering::SeqCst);
}

/// Disable team memory sync
pub fn disable_team_memory() {
    TEAM_MEMORY_ENABLED.store(false, Ordering::SeqCst);
}

/// Get last sync error (thread-safe)
static LAST_SYNC_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// Set last sync error
pub fn set_last_sync_error(error: Option<String>) {
    *LAST_SYNC_ERROR.lock().unwrap() = error;
}

/// Get last sync error
pub fn get_last_sync_error() -> Option<String> {
    LAST_SYNC_ERROR.lock().unwrap().clone()
}

/// Reset team memory globals for testing
pub fn reset_team_memory_for_testing() {
    disable_team_memory();
    *LAST_SYNC_ERROR.lock().unwrap() = None;
}

// ─── Tests ─────────────────────────────────────────────────────
