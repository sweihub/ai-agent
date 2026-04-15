// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/filesApi.ts
//! Files API client for managing files
//!
//! This module provides functionality to download and upload files to Anthropic Public Files API.
//! Used by the Claude Code agent to download file attachments at session startup.
//!
//! API Reference: https://docs.anthropic.com/en/api/files-content

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::fs;

/// Files API beta header
const FILES_API_BETA_HEADER: &str = "files-api-2025-04-14,oauth-2025-04-20";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Maximum file size: 500MB
const MAX_FILE_SIZE_BYTES: usize = 500 * 1024 * 1024;

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 500;

/// Default concurrency limit for parallel downloads
const DEFAULT_CONCURRENCY: usize = 5;

/// Get default API base URL
fn get_default_api_base_url() -> String {
    std::env::var("AI_CODE_BASE_URL")
        .or_else(|_| std::env::var("AI_CODE_API_BASE_URL"))
        .unwrap_or_else(|_| "https://api.anthropic.com".to_string())
}

/// Log debug message
fn log_debug(message: &str) {
    log::debug!("[files-api] {}", message);
}

/// Log debug error message
fn log_debug_error(message: &str) {
    log::error!("[files-api] {}", message);
}

/// Sleep for specified milliseconds
async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

/// File specification parsed from CLI args
/// Format: --file=<file_id>:<relative_path>
#[derive(Debug, Clone)]
pub struct File {
    pub file_id: String,
    pub relative_path: String,
}

/// Configuration for the files API client
#[derive(Debug, Clone)]
pub struct FilesApiConfig {
    /// OAuth token for authentication (from session JWT)
    pub oauth_token: String,
    /// Base URL for the API (default: https://api.anthropic.com)
    pub base_url: Option<String>,
    /// Session ID for creating session-specific directories
    pub session_id: String,
}

/// Result of a file download operation
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub file_id: String,
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
    pub bytes_written: Option<usize>,
}

/// Normalizes a relative path, strips redundant prefixes, and builds the full
/// download path under {basePath}/{session_id}/uploads/.
/// Returns None if the path is invalid (e.g., path traversal).
pub fn build_download_path(
    base_path: &str,
    session_id: &str,
    relative_path: &str,
) -> Option<PathBuf> {
    // Check for path traversal in original path
    let normalized_original = std::path::Path::new(relative_path)
        .components()
        .fold(PathBuf::new(), |mut acc, c| {
            match c {
                std::path::Component::Normal(p) => acc.push(p),
                std::path::Component::ParentDir => {
                    acc.pop();
                }
                _ => {}
            }
            acc
        });

    // Check for path traversal - original path shouldn't start with ".."
    // after normalization, if we went up directories
    let normalized_str = normalized_original.to_string_lossy().to_string();
    if normalized_str.starts_with("..") || relative_path.starts_with("..") {
        log_debug_error(&format!(
            "Invalid file path: {}. Path must not traverse above workspace",
            relative_path
        ));
        return None;
    }

    let uploads_base = PathBuf::from(base_path)
        .join(session_id)
        .join("uploads");

    let redundant_prefixes = vec![
        uploads_base.to_string_lossy().to_string() + std::path::MAIN_SEPARATOR_STR,
        std::path::MAIN_SEPARATOR_STR.to_string() + "uploads" + std::path::MAIN_SEPARATOR_STR,
    ];

    let clean_path = redundant_prefixes
        .iter()
        .find_map(|p| {
            if normalized_str.starts_with(p) {
                Some(normalized_str[p.len()..].to_string())
            } else {
                None
            }
        })
        .unwrap_or(normalized_str);

    Some(uploads_base.join(clean_path))
}

/// Downloads a file and saves it to the session-specific workspace directory
pub async fn download_and_save_file(
    attachment: &File,
    config: &FilesApiConfig,
    base_path: &str,
) -> DownloadResult {
    let file_id = attachment.file_id.clone();
    let relative_path = attachment.relative_path.clone();

    let full_path = match build_download_path(base_path, &config.session_id, &relative_path) {
        Some(p) => p,
        None => {
            return DownloadResult {
                file_id: file_id.clone(),
                path: String::new(),
                success: false,
                error: Some(format!("Invalid file path: {}", relative_path)),
                bytes_written: None,
            };
        }
    };

    let full_path_str = full_path.to_string_lossy().to_string();
    let base_url = config.base_url.clone().unwrap_or_else(get_default_api_base_url);
    let url = format!("{}/v1/files/{}/content", base_url, file_id);

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(60000))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return DownloadResult {
                file_id,
                path: full_path_str,
                success: false,
                error: Some(e.to_string()),
                bytes_written: None,
            };
        }
    };

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", config.oauth_token).parse().unwrap(),
    );
    headers.insert(
        "anthropic-version",
        ANTHROPIC_VERSION.parse().unwrap(),
    );
    headers.insert("anthropic-beta", FILES_API_BETA_HEADER.parse().unwrap());

    // Download with retries
    let mut last_error = String::new();
    for attempt in 1..=MAX_RETRIES {
        let response = client.get(&url).headers(headers.clone()).send().await;

        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    last_error = match resp.status() {
                        s if s == reqwest::StatusCode::NOT_FOUND => format!("File not found: {}", file_id),
                        s if s == reqwest::StatusCode::UNAUTHORIZED => "Authentication failed".to_string(),
                        s if s == reqwest::StatusCode::FORBIDDEN => format!("Access denied to file: {}", file_id),
                        _ => format!("status {}", resp.status()),
                    };
                    // Non-retriable for 4xx
                    if resp.status().is_client_error() {
                        return DownloadResult {
                            file_id,
                            path: full_path_str,
                            success: false,
                            error: Some(last_error),
                            bytes_written: None,
                        };
                    }
                } else {
                    // Success - read content
                    match resp.bytes().await {
                        Ok(bytes) => {
                            let content = bytes.to_vec();
                            // Ensure the parent directory exists
                            if let Some(parent) = full_path.parent() {
                                if let Err(e) = fs::create_dir_all(parent).await {
                                    log_debug_error(&format!("Failed to create directory: {}", e));
                                    return DownloadResult {
                                        file_id,
                                        path: full_path_str,
                                        success: false,
                                        error: Some(e.to_string()),
                                        bytes_written: None,
                                    };
                                }
                            }
                            // Write the file
                            match fs::write(&full_path, &content).await {
                                Ok(_) => {
                                    log_debug(&format!(
                                        "Saved file {} to {} ({} bytes)",
                                        file_id,
                                        full_path.display(),
                                        content.len()
                                    ));
                                    return DownloadResult {
                                        file_id,
                                        path: full_path_str,
                                        success: true,
                                        bytes_written: Some(content.len()),
                                        error: None,
                                    };
                                }
                                Err(e) => {
                                    log_debug_error(&format!("Failed to write file {}: {}", file_id, e));
                                    return DownloadResult {
                                        file_id,
                                        path: full_path_str,
                                        success: false,
                                        error: Some(e.to_string()),
                                        bytes_written: None,
                                    };
                                }
                            }
                        }
                        Err(e) => {
                            last_error = e.to_string();
                        }
                    }
                }
            }
            Err(e) => {
                last_error = e.to_string();
            }
        }

        if attempt < MAX_RETRIES {
            let delay_ms = BASE_DELAY_MS * 2u64.pow(attempt - 1);
            log_debug(&format!(
                "Download file {} attempt {}/{} failed: {}, retrying in {}ms",
                file_id, attempt, MAX_RETRIES, last_error, delay_ms
            ));
            sleep_ms(delay_ms).await;
        }
    }

    log_debug_error(&format!("Failed to download file {}: {}", file_id, last_error));
    DownloadResult {
        file_id,
        path: full_path_str,
        success: false,
        error: Some(format!("{} after {} attempts", last_error, MAX_RETRIES)),
        bytes_written: None,
    }
}

/// Downloads all file attachments for a session
pub async fn download_session_files(
    files: Vec<File>,
    config: FilesApiConfig,
    base_path: &str,
    _concurrency: usize,
) -> Vec<DownloadResult> {
    if files.is_empty() {
        return Vec::new();
    }

    log_debug(&format!(
        "Downloading {} file(s) for session {}",
        files.len(),
        config.session_id
    ));

    let start_time = std::time::Instant::now();
    let base_path_owned = base_path.to_string();

    // Sequential for now
    let file_count = files.len();
    let mut results = Vec::with_capacity(file_count);
    for file in files {
        let result = download_and_save_file(&file, &config, &base_path_owned).await;
        results.push(result);
    }

    let elapsed_ms = start_time.elapsed().as_millis() as u64;
    let success_count = results.iter().filter(|r| r.success).count();
    log_debug(&format!(
        "Downloaded {}/{} file(s) in {}ms",
        success_count,
        file_count,
        elapsed_ms
    ));

    results
}

// ============================================================================
// Upload Functions (BYOC mode)
// ============================================================================

/// Result of a file upload operation
#[derive(Debug, Clone)]
pub enum UploadResult {
    Success {
        path: String,
        file_id: String,
        size: usize,
    },
    Failure {
        path: String,
        error: String,
    },
}

/// Upload a single file to the Files API (BYOC mode)
pub async fn upload_file(
    file_path: &str,
    relative_path: &str,
    config: &FilesApiConfig,
) -> UploadResult {
    let base_url = config.base_url.clone().unwrap_or_else(get_default_api_base_url);
    let url = format!("{}/v1/files", base_url);

    log_debug(&format!("Uploading file {} as {}", file_path, relative_path));

    // Read file content first
    let content = match fs::read(file_path).await {
        Ok(c) => c,
        Err(e) => {
            return UploadResult::Failure {
                path: relative_path.to_string(),
                error: e.to_string(),
            };
        }
    };

    let file_size = content.len();

    if file_size > MAX_FILE_SIZE_BYTES {
        return UploadResult::Failure {
            path: relative_path.to_string(),
            error: format!(
                "File exceeds maximum size of {} bytes (actual: {})",
                MAX_FILE_SIZE_BYTES, file_size
            ),
        };
    }

    // Use UUID for boundary
    let boundary = format!("----FormBoundary{}", uuid::Uuid::new_v4());
    let filename = std::path::Path::new(relative_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| relative_path.to_string());

    // Build the multipart body
    let mut body_parts: Vec<Vec<u8>> = Vec::new();

    // File part
    body_parts.push(
        format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
            boundary, filename
        )
        .as_bytes()
        .to_vec(),
    );
    body_parts.push(content);
    body_parts.push(b"\r\n".to_vec());

    // Purpose part
    body_parts.push(
        format!(
            "--{}\r\nContent-Disposition: form-data; name=\"purpose\"\r\n\r\nuser_data\r\n",
            boundary
        )
        .as_bytes()
        .to_vec(),
    );

    // End boundary
    body_parts.push(format!("--{}--\r\n", boundary).as_bytes().to_vec());

    let body: Vec<u8> = body_parts.into_iter().flatten().collect();

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(120000))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return UploadResult::Failure {
                path: relative_path.to_string(),
                error: e.to_string(),
            };
        }
    };

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", config.oauth_token).parse().unwrap(),
    );
    headers.insert(
        "anthropic-version",
        ANTHROPIC_VERSION.parse().unwrap(),
    );
    headers.insert("anthropic-beta", FILES_API_BETA_HEADER.parse().unwrap());
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        format!("multipart/form-data; boundary={}", boundary)
            .parse()
            .unwrap(),
    );
    headers.insert(
        reqwest::header::CONTENT_LENGTH,
        body.len().to_string().parse().unwrap(),
    );

    let mut last_error = String::new();
    for attempt in 1..=MAX_RETRIES {
        let response = client.post(&url).headers(headers.clone()).body(body.clone()).send().await;

        match response {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::OK || resp.status() == reqwest::StatusCode::CREATED {
                    // Try to get the file ID from response
                    match resp.json::<serde_json::Value>().await {
                        Ok(data) => {
                            let file_id_opt = data
                                .get("id")
                                .and_then(|v| v.as_str())
                                .map(String::from);

                            if let Some(file_id) = file_id_opt {
                                log_debug(&format!(
                                    "Uploaded file {} -> {} ({} bytes)",
                                    file_path, file_id, file_size
                                ));
                                return UploadResult::Success {
                                    path: relative_path.to_string(),
                                    file_id,
                                    size: file_size,
                                };
                            } else {
                                last_error = "Upload succeeded but no file ID returned".to_string();
                            }
                        }
                        Err(e) => {
                            last_error = e.to_string();
                        }
                    }
                } else if resp.status().is_client_error() {
                    // Non-retriable errors for 4xx
                    let error_msg = match resp.status() {
                        s if s == reqwest::StatusCode::UNAUTHORIZED => {
                            "Authentication failed: invalid or missing API key".to_string()
                        }
                        s if s == reqwest::StatusCode::FORBIDDEN => {
                            "Access denied for upload".to_string()
                        }
                        s if s == reqwest::StatusCode::PAYLOAD_TOO_LARGE => {
                            "File too large for upload".to_string()
                        }
                        _ => format!("status {}", resp.status()),
                    };
                    return UploadResult::Failure {
                        path: relative_path.to_string(),
                        error: error_msg,
                    };
                } else {
                    last_error = format!("status {}", resp.status());
                }
            }
            Err(e) => {
                last_error = e.to_string();
            }
        }

        if attempt < MAX_RETRIES {
            let delay_ms = BASE_DELAY_MS * 2u64.pow(attempt - 1);
            log_debug(&format!(
                "Upload file {} attempt {}/{} failed: {}, retrying in {}ms",
                relative_path, attempt, MAX_RETRIES, last_error, delay_ms
            ));
            sleep_ms(delay_ms).await;
        }
    }

    UploadResult::Failure {
        path: relative_path.to_string(),
        error: format!("{} after {} attempts", last_error, MAX_RETRIES),
    }
}

/// Upload multiple files (BYOC mode)
pub async fn upload_session_files(
    files: Vec<(String, String)>, // (path, relative_path)
    config: FilesApiConfig,
    _concurrency: usize,
) -> Vec<UploadResult> {
    if files.is_empty() {
        return Vec::new();
    }

    log_debug(&format!(
        "Uploading {} file(s) for session {}",
        files.len(),
        config.session_id
    ));

    let start_time = std::time::Instant::now();

    // Sequential for now
    let file_count = files.len();
    let mut results = Vec::with_capacity(file_count);
    for (path, relative_path) in files {
        let result = upload_file(&path, &relative_path, &config).await;
        results.push(result);
    }

    let elapsed_ms = start_time.elapsed().as_millis() as u64;
    let success_count = results
        .iter()
        .filter(|r| matches!(r, UploadResult::Success { .. }))
        .count();
    log_debug(&format!(
        "Uploaded {}/{} file(s) in {}ms",
        success_count,
        file_count,
        elapsed_ms
    ));

    results
}

// ============================================================================
// List Files Functions (1P/Cloud mode)
// ============================================================================

/// File metadata returned from list_files_created_after
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub filename: String,
    pub file_id: String,
    pub size: usize,
}

/// List files created after a given timestamp (1P/Cloud mode).
pub async fn list_files_created_after(
    after_created_at: &str,
    config: &FilesApiConfig,
) -> Result<Vec<FileMetadata>, String> {
    let base_url = config.base_url.clone().unwrap_or_else(get_default_api_base_url);
    let url = format!("{}/v1/files", base_url);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", config.oauth_token).parse().unwrap(),
    );
    headers.insert(
        "anthropic-version",
        ANTHROPIC_VERSION.parse().unwrap(),
    );
    headers.insert("anthropic-beta", FILES_API_BETA_HEADER.parse().unwrap());

    log_debug(&format!("Listing files created after {}", after_created_at));

    let mut all_files: Vec<FileMetadata> = Vec::new();
    let mut after_id: Option<String> = None;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(60000))
        .build()
        .map_err(|e| e.to_string())?;

    loop {
        let mut params = HashMap::new();
        params.insert("after_created_at", after_created_at.to_string());

        if let Some(ref aid) = after_id {
            params.insert("after_id", aid.clone());
        }

        let response = client
            .get(&url)
            .headers(headers.clone())
            .query(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status() != reqwest::StatusCode::OK {
            if response.status().is_client_error() {
                return Ok(Vec::new());
            }
            return Err(format!("status {}", response.status()));
        }

        let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        let files: Vec<FileMetadata> = data
            .get("data")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|f| FileMetadata {
                        filename: f.get("filename").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        file_id: f.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        size: f.get("size_bytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                    })
                    .collect()
            })
            .unwrap_or_default();

        all_files.extend(files);

        let has_more = data.get("has_more").and_then(|v| v.as_bool()).unwrap_or(false);
        if !has_more {
            break;
        }

        if let Some(last_file) = all_files.last() {
            after_id = Some(last_file.file_id.clone());
        } else {
            break;
        }
    }

    log_debug(&format!(
        "Listed {} files created after {}",
        all_files.len(),
        after_created_at
    ));
    Ok(all_files)
}

// ============================================================================
// Parse Functions
// ============================================================================

/// Parse file attachment specs from CLI arguments
/// Format: <file_id>:<relative_path>
pub fn parse_file_specs(file_specs: Vec<String>) -> Vec<File> {
    let mut files = Vec::new();

    // Sandbox-gateway may pass multiple specs as a single space-separated string
    let expanded_specs: Vec<String> = file_specs
        .into_iter()
        .flat_map(|s| {
            s.split(' ')
                .filter(|s2| !s2.is_empty())
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .collect();

    for spec in expanded_specs {
        let Some(colon_index) = spec.find(':') else {
            continue;
        };

        let file_id = spec[..colon_index].to_string();
        let relative_path = spec[colon_index + 1..].to_string();

        if file_id.is_empty() || relative_path.is_empty() {
            log_debug_error(&format!(
                "Invalid file spec: {}. Both file_id and path are required",
                spec
            ));
            continue;
        }

        files.push(File {
            file_id,
            relative_path,
        });
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_download_path_simple() {
        let result = build_download_path("/workspace", "session123", "file.txt");
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.to_string_lossy().ends_with("session123/uploads/file.txt"));
    }

    #[test]
    fn test_build_download_path_traversal() {
        let result = build_download_path("/workspace", "session123", "../etc/passwd");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_file_specs_simple() {
        let files = parse_file_specs(vec!["file_123:path/to/file.txt".to_string()]);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_id, "file_123");
        assert_eq!(files[0].relative_path, "path/to/file.txt");
    }

    #[test]
    fn test_parse_file_specs_multiple() {
        let files = parse_file_specs(vec![
            "file_1:path1.txt".to_string(),
            "file_2:path2.txt".to_string(),
        ]);
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_parse_file_specs_invalid() {
        let files = parse_file_specs(vec!["invalid_spec".to_string()]);
        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_file_specs_spaced() {
        let files = parse_file_specs(vec!["file_1:path1.txt file_2:path2.txt".to_string()]);
        assert_eq!(files.len(), 2);
    }
}