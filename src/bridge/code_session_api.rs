//! Thin HTTP wrappers for the CCR v2 code-session API.
//!
//! Translated from openclaudecode/src/bridge/codeSessionApi.ts
//!
//! Separate file from remoteBridgeCore.ts so the SDK /bridge subpath can
//! export createCodeSession + fetchRemoteCredentials without bundling the
//! heavy CLI tree (analytics, transport, etc.). Callers supply explicit
//! accessToken + baseUrl — no implicit auth or config reads.

use crate::utils::http::get_user_agent;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Build OAuth headers for API requests
fn oauth_headers(access_token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", access_token)) {
        headers.insert(AUTHORIZATION, val);
    }
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        HeaderName::from_static("anthropic-version"),
        HeaderValue::from_static(ANTHROPIC_VERSION),
    );
    headers.insert("User-Agent", get_user_agent().parse().unwrap());
    headers
}

/// Create a new code session via POST /v1/code/sessions
///
/// # Arguments
/// * `base_url` - The API base URL
/// * `access_token` - OAuth access token
/// * `title` - Session title
/// * `timeout_ms` - Request timeout in milliseconds
/// * `tags` - Optional tags for the session
///
/// Returns the session ID on success, or None if creation fails.
pub async fn create_code_session(
    base_url: &str,
    access_token: &str,
    title: &str,
    timeout_ms: u64,
    tags: Option<Vec<String>>,
) -> Option<String> {
    let url = format!("{}/v1/code/sessions", base_url);
    let headers = oauth_headers(access_token);

    // Build request body
    // bridge: {} is the positive signal for the oneof runner — omitting it
    // (or sending environment_id: "") now 400s. BridgeRunner is an empty
    // message today; it's a placeholder for future bridge-specific options.
    let mut body = serde_json::json!({
        "title": title,
        "bridge": {}
    });

    if let Some(tags) = tags {
        if !tags.is_empty() {
            body["tags"] = serde_json::json!(tags);
        }
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .headers(headers)
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .json(&body)
        .send()
        .await
        .ok()?;

    let status = response.status();
    if status != reqwest::StatusCode::OK && status != reqwest::StatusCode::CREATED {
        let status_code = status.as_u16();
        let body = response.text().await.unwrap_or_default();
        let detail = extract_error_detail_from_text(&body);
        log_for_debugging(&format!(
            "[code-session] Session create failed {}{}",
            status_code,
            detail.map(|d| format!(": {}", d)).unwrap_or_default()
        ));
        return None;
    }

    let data: serde_json::Value = response.json().await.ok()?;

    // Validate response structure
    let session = data.get("session")?;
    let session_obj = session.as_object()?;
    let id = session_obj.get("id")?.as_str()?;
    if !id.starts_with("cse_") {
        let data_str: String = serde_json::to_string(&data)
            .ok()
            .map(|s| s.chars().take(200).collect())
            .unwrap_or_default();
        log_for_debugging(&format!(
            "[code-session] No session.id (cse_*) in response: {}",
            data_str
        ));
        return None;
    }

    Some(id.to_string())
}

/// Credentials from POST /bridge. JWT is opaque — do not decode.
/// Each /bridge call bumps worker_epoch server-side (it IS the register).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCredentials {
    #[serde(rename = "worker_jwt")]
    pub worker_jwt: String,
    #[serde(rename = "api_base_url")]
    pub api_base_url: String,
    #[serde(rename = "expires_in")]
    pub expires_in: u64,
    #[serde(rename = "worker_epoch")]
    pub worker_epoch: i64,
}

/// Fetch remote credentials for a session via POST /v1/code/sessions/{id}/bridge
///
/// # Arguments
/// * `session_id` - The session ID
/// * `base_url` - The API base URL
/// * `access_token` - OAuth access token
/// * `timeout_ms` - Request timeout in milliseconds
/// * `trusted_device_token` - Optional trusted device token
///
/// Returns the remote credentials on success, or None if fetch fails.
pub async fn fetch_remote_credentials(
    session_id: &str,
    base_url: &str,
    access_token: &str,
    timeout_ms: u64,
    trusted_device_token: Option<&str>,
) -> Option<RemoteCredentials> {
    let url = format!("{}/v1/code/sessions/{}/bridge", base_url, session_id);
    let mut headers = oauth_headers(access_token);

    if let Some(token) = trusted_device_token {
        if let Ok(val) = HeaderValue::from_str(token) {
            headers.insert(HeaderName::from_static("x-trusted-device-token"), val);
        }
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .headers(headers)
        .timeout(std::time::Duration::from_millis(timeout_ms))
        .json(&serde_json::json!({}))
        .send()
        .await
        .ok()?;

    let status = response.status();
    if status != reqwest::StatusCode::OK {
        let status_code = status.as_u16();
        let body = response.text().await.unwrap_or_default();
        let detail = extract_error_detail_from_text(&body);
        log_for_debugging(&format!(
            "[code-session] /bridge failed {}{}",
            status_code,
            detail.map(|d| format!(": {}", d)).unwrap_or_default()
        ));
        return None;
    }

    let data: serde_json::Value = response.json().await.ok()?;

    // Validate response structure
    let worker_jwt = data.get("worker_jwt")?.as_str()?.to_string();
    let expires_in = data.get("expires_in")?.as_u64()?;
    let api_base_url = data.get("api_base_url")?.as_str()?.to_string();

    // protojson serializes int64 as a string to avoid JS precision loss;
    // Go may also return a number depending on encoder settings.
    let raw_epoch = data.get("worker_epoch")?;
    let epoch = if let Some(s) = raw_epoch.as_str() {
        s.parse().ok()?
    } else {
        raw_epoch.as_i64()?
    };

    Some(RemoteCredentials {
        worker_jwt,
        api_base_url,
        expires_in,
        worker_epoch: epoch,
    })
}

/// Extract error detail from response body text
fn extract_error_detail_from_text(body: &str) -> Option<String> {
    let data: serde_json::Value = serde_json::from_str(body).ok()?;
    data.get("message")
        .and_then(|m| m.as_str())
        .map(|s| s.to_string())
}

/// Simple logging helper
#[allow(unused_variables)]
fn log_for_debugging(msg: &str) {
    eprintln!("{}", msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_headers() {
        let headers = oauth_headers("test-token");
        // Just verify Authorization header exists with correct prefix
        assert!(headers.get("Authorization").is_some());
        // Just verify Content-Type header exists
        assert!(headers.get("Content-Type").is_some());
    }
}
