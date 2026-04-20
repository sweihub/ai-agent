//! Work secret handling and session ID utilities.
//!
//! Translated from openclaudecode/src/bridge/workSecret.ts

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use crate::utils::http::get_user_agent;

#[cfg(feature = "reqwest")]
use reqwest;

/// Work secret structure decoded from base64url-encoded JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSecret {
    pub version: u32,
    pub session_ingress_token: String,
    pub api_base_url: String,
    pub sources: Vec<WorkSource>,
    pub auth: Vec<WorkAuth>,
    #[serde(default)]
    pub claude_code_args: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub mcp_config: Option<serde_json::Value>,
    #[serde(default)]
    pub environment_variables: Option<std::collections::HashMap<String, String>>,
    /// Server-driven CCR v2 selector. Set when the session was created
    /// via the v2 compat layer.
    #[serde(default)]
    pub use_code_sessions: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default)]
    pub git_info: Option<GitInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    #[serde(rename = "type")]
    pub git_type: String,
    pub repo: String,
    #[serde(default)]
    pub r#ref: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    pub token: String,
}

/// Decode a base64url-encoded work secret and validate its version.
pub fn decode_work_secret(secret: &str) -> Result<WorkSecret, String> {
    let json = URL_SAFE_NO_PAD
        .decode(secret)
        .map_err(|e| format!("Failed to decode base64url: {}", e))?;

    let parsed: serde_json::Value =
        serde_json::from_slice(&json).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if let Some(obj) = parsed.as_object() {
        let version = obj.get("version").and_then(|v| v.as_u64()).unwrap_or(0);

        if version != 1 {
            return Err(format!(
                "Unsupported work secret version: {}",
                obj.get("version")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            ));
        }

        // Validate required fields
        let session_ingress_token = obj
            .get("session_ingress_token")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .ok_or("Invalid work secret: missing or empty session_ingress_token")?;

        let api_base_url = obj
            .get("api_base_url")
            .and_then(|v| v.as_str())
            .ok_or("Invalid work secret: missing api_base_url")?;

        let work_secret = WorkSecret {
            version: version as u32,
            session_ingress_token: session_ingress_token.to_string(),
            api_base_url: api_base_url.to_string(),
            sources: obj
                .get("sources")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            auth: obj
                .get("auth")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            claude_code_args: obj
                .get("claude_code_args")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            mcp_config: obj.get("mcp_config").cloned(),
            environment_variables: obj
                .get("environment_variables")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            use_code_sessions: obj.get("use_code_sessions").and_then(|v| v.as_bool()),
        };

        Ok(work_secret)
    } else {
        Err("Invalid work secret: not an object".to_string())
    }
}

/// Build a WebSocket SDK URL from the API base URL and session ID.
/// Strips the HTTP(S) protocol and constructs a ws(s):// ingress URL.
///
/// Uses /v2/ for localhost (direct to session-ingress, no Envoy rewrite)
/// and /v1/ for production (Envoy rewrites /v1/ -> /v2/).
pub fn build_sdk_url(api_base_url: &str, session_id: &str) -> String {
    let is_localhost = api_base_url.contains("localhost") || api_base_url.contains("127.0.0.1");
    let protocol = if is_localhost { "ws" } else { "wss" };
    let version = if is_localhost { "v2" } else { "v1" };
    let host = api_base_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');

    format!(
        "{}://{}/{}/session_ingress/ws/{}",
        protocol, host, version, session_id
    )
}

/// Compare two session IDs regardless of their tagged-ID prefix.
///
/// Tagged IDs have the form {tag}_{body} or {tag}_staging_{body}, where the
/// body encodes a UUID. CCR v2's compat layer returns `session_*` to v1 API
/// clients but the infrastructure layer uses `cse_*`. Both have the same
/// underlying UUID.
pub fn same_session_id(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }

    // The body is everything after the last underscore — this handles both
    // `{tag}_{body}` and `{tag}_staging_{body}`.
    let a_body = a.split('_').last().unwrap_or("");
    let b_body = b.split('_').last().unwrap_or("");

    // Guard against IDs with no underscore (bare UUIDs).
    // Require a minimum length to avoid accidental matches on short suffixes.
    a_body.len() >= 4 && a_body == b_body
}

/// Build a CCR v2 session URL from the API base URL and session ID.
/// Returns an HTTP(S) URL (not ws://) and points at /v1/code/sessions/{id}.
pub fn build_ccr_v2_sdk_url(api_base_url: &str, session_id: &str) -> String {
    let base = api_base_url.trim_end_matches('/');
    format!("{}/v1/code/sessions/{}", base, session_id)
}

/// Register this bridge as the worker for a CCR v2 session.
/// Returns the worker_epoch, which must be passed to the child CC process.
pub async fn register_worker(session_url: &str, access_token: &str) -> Result<u64, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/worker/register", session_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .header("User-Agent", get_user_agent())
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let raw = data.get("worker_epoch");

    let epoch = match raw {
        Some(v) if v.is_number() => v.as_u64(),
        Some(v) if v.is_string() => v.as_str().and_then(|s| s.parse().ok()),
        _ => None,
    };

    epoch.ok_or_else(|| {
        format!(
            "register_worker: invalid worker_epoch in response: {}",
            data
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sdk_url() {
        // Production
        assert_eq!(
            build_sdk_url("https://api.anthropic.com", "session_abc"),
            "wss://api.anthropic.com/v1/session_ingress/ws/session_abc"
        );

        // Localhost
        assert_eq!(
            build_sdk_url("http://localhost:8080", "session_abc"),
            "ws://localhost:8080/v2/session_ingress/ws/session_abc"
        );
    }

    #[test]
    fn test_same_session_id() {
        // Same ID
        assert!(same_session_id("session_abc123", "session_abc123"));

        // Same UUID with different tags
        assert!(same_session_id("cse_abc123", "session_abc123"));

        // Different UUIDs
        assert!(!same_session_id("session_abc123", "session_xyz789"));

        // Staging format
        assert!(same_session_id(
            "cse_staging_abc123",
            "session_staging_abc123"
        ));
    }

    #[test]
    fn test_build_ccr_v2_sdk_url() {
        assert_eq!(
            build_ccr_v2_sdk_url("https://api.anthropic.com", "session_abc"),
            "https://api.anthropic.com/v1/code/sessions/session_abc"
        );
    }

    #[test]
    fn test_decode_work_secret() {
        let secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
            r#"{"version":1,"session_ingress_token":"tok123","api_base_url":"https://api.example.com","sources":[],"auth":[]}"#
        );

        let decoded = decode_work_secret(&secret).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.session_ingress_token, "tok123");
        assert_eq!(decoded.api_base_url, "https://api.example.com");
    }
}
