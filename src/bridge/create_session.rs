//! Bridge session creation and management.
//!
//! Translated from openclaudecode/src/bridge/createSession.ts
//!
//! Functions for creating, fetching, archiving, and updating bridge sessions.

use crate::constants::env::ai;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

/// Git source for session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: String,
    pub revision: Option<String>,
}

/// Git outcome for session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOutcome {
    #[serde(rename = "type")]
    pub outcome_type: String,
    #[serde(rename = "git_info")]
    pub git_info: GitInfo,
}

/// Git info for GitHub repositories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    #[serde(rename = "type")]
    pub info_type: String,
    pub repo: String,
    pub branches: Vec<String>,
}

/// Session event wrapper for POST /v1/sessions endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Session context for session creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub sources: Vec<GitSource>,
    pub outcomes: Vec<GitOutcome>,
    pub model: Option<String>,
}

/// Create a session on a bridge environment via POST /v1/sessions.
///
/// Used by both `claude remote-control` (empty session so the user has somewhere to
/// type immediately) and `/remote-control` (session pre-populated with conversation
/// history).
///
/// Returns the session ID on success, or None if creation fails (non-fatal).
pub async fn create_bridge_session(
    environment_id: &str,
    title: Option<&str>,
    events: Vec<SessionEvent>,
    git_repo_url: Option<&str>,
    branch: Option<&str>,
    base_url: Option<&str>,
    get_access_token: Option<&dyn Fn() -> Option<String>>,
    permission_mode: Option<&str>,
) -> Option<String> {
    // Get access token
    let access_token = get_access_token
        .and_then(|f| f())
        .or_else(|| crate::bridge::get_bridge_access_token());

    let access_token = match access_token {
        Some(t) => t,
        None => {
            log_for_debugging("[bridge] No access token for session creation");
            return None;
        }
    };

    // Get organization UUID
    let org_uuid = get_organization_uuid();
    let org_uuid = match org_uuid {
        Some(uuid) => uuid,
        None => {
            log_for_debugging("[bridge] No org UUID for session creation");
            return None;
        }
    };

    // Build git source and outcome context
    let (git_source, git_outcome) = if let Some(repo_url) = git_repo_url {
        build_git_context(repo_url, branch)
    } else {
        (None, None)
    };

    // Build request body
    let mut request_body = serde_json::json!({
        "events": events,
        "session_context": {
            "sources": git_source.map(|s| vec![s]).unwrap_or_default(),
            "outcomes": git_outcome.map(|o| vec![o]).unwrap_or_default(),
            "model": get_main_loop_model(),
        },
        "environment_id": environment_id,
        "source": "remote-control",
    });

    if let Some(t) = title {
        request_body["title"] = serde_json::json!(t);
    }

    if let Some(mode) = permission_mode {
        request_body["permission_mode"] = serde_json::json!(mode);
    }

    let headers = build_oauth_headers(&access_token, &org_uuid);

    let url = format!("{}/v1/sessions", base_url.unwrap_or(&get_oauth_config()));

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .ok()?;

    let status = response.status();
    if status != reqwest::StatusCode::OK && status != reqwest::StatusCode::CREATED {
        let status_code = status.as_u16();
        let body = response.text().await.unwrap_or_default();
        let detail = extract_error_detail_from_text(&body);
        log_for_debugging(&format!(
            "[bridge] Session creation failed with status {}{}",
            status_code,
            detail.map(|d| format!(": {}", d)).unwrap_or_default()
        ));
        return None;
    }

    let session_data: serde_json::Value = response.json().await.ok()?;

    let session_id = session_data.get("id")?.as_str()?.to_string();
    Some(session_id)
}

/// Fetch a bridge session via GET /v1/sessions/{id}.
///
/// Returns the session's environment_id (for `--session-id` resume) and title.
/// Uses the same org-scoped headers as create/archive — the environments-level
/// client in bridgeApi.ts uses a different beta header and no org UUID, which
/// makes the Sessions API return 404.
pub async fn get_bridge_session(
    session_id: &str,
    base_url: Option<&str>,
    get_access_token: Option<&dyn Fn() -> Option<String>>,
) -> Option<BridgeSessionInfo> {
    // Get access token
    let access_token = get_access_token
        .and_then(|f| f())
        .or_else(|| crate::bridge::get_bridge_access_token());

    let access_token = match access_token {
        Some(t) => t,
        None => {
            log_for_debugging("[bridge] No access token for session fetch");
            return None;
        }
    };

    // Get organization UUID
    let org_uuid = get_organization_uuid();
    let org_uuid = match org_uuid {
        Some(uuid) => uuid,
        None => {
            log_for_debugging("[bridge] No org UUID for session fetch");
            return None;
        }
    };

    let headers = build_oauth_headers(&access_token, &org_uuid);

    let url = format!(
        "{}/v1/sessions/{}",
        base_url.unwrap_or(&get_oauth_config()),
        session_id
    );

    log_for_debugging(&format!("[bridge] Fetching session {}", session_id));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .headers(headers)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;

    let status = response.status();
    if status != reqwest::StatusCode::OK {
        let status_code = status.as_u16();
        let body = response.text().await.unwrap_or_default();
        let detail = extract_error_detail_from_text(&body);
        log_for_debugging(&format!(
            "[bridge] Session fetch failed with status {}{}",
            status_code,
            detail.map(|d| format!(": {}", d)).unwrap_or_default()
        ));
        return None;
    }

    let data: serde_json::Value = response.json().await.ok()?;
    Some(BridgeSessionInfo {
        environment_id: data
            .get("environment_id")
            .and_then(|v| v.as_str())
            .map(String::from),
        title: data.get("title").and_then(|v| v.as_str()).map(String::from),
    })
}

/// Bridge session info
#[derive(Debug, Clone)]
pub struct BridgeSessionInfo {
    pub environment_id: Option<String>,
    pub title: Option<String>,
}

/// Archive a bridge session via POST /v1/sessions/{id}/archive.
///
/// The CCR server never auto-archives sessions — archival is always an
/// explicit client action. Both `claude remote-control` (standalone bridge) and the
/// always-on `/remote-control` REPL bridge call this during shutdown to archive any
/// sessions that are still alive.
///
/// The archive endpoint accepts sessions in any status (running, idle,
/// requires_action, pending) and returns 409 if already archived, making
/// it safe to call even if the server-side runner already archived the
/// session.
///
/// Callers must handle errors — this function has no try/catch; 5xx,
/// timeouts, and network errors throw. Archival is best-effort during
/// cleanup; call sites wrap with .catch().
pub async fn archive_bridge_session(
    session_id: &str,
    base_url: Option<&str>,
    get_access_token: Option<&dyn Fn() -> Option<String>>,
    timeout_ms: Option<u64>,
) -> Result<(), String> {
    // Get access token
    let access_token = get_access_token
        .and_then(|f| f())
        .or_else(|| crate::bridge::get_bridge_access_token());

    let access_token = match access_token {
        Some(t) => t,
        None => {
            log_for_debugging("[bridge] No access token for session archive");
            return Err("No access token".to_string());
        }
    };

    // Get organization UUID
    let org_uuid = get_organization_uuid();
    let org_uuid = match org_uuid {
        Some(uuid) => uuid,
        None => {
            log_for_debugging("[bridge] No org UUID for session archive");
            return Err("No org UUID".to_string());
        }
    };

    let headers = build_oauth_headers(&access_token, &org_uuid);

    let url = format!(
        "{}/v1/sessions/{}/archive",
        base_url.unwrap_or(&get_oauth_config()),
        session_id
    );

    log_for_debugging(&format!("[bridge] Archiving session {}", session_id));

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .headers(headers)
        .timeout(std::time::Duration::from_millis(
            timeout_ms.unwrap_or(10_000),
        ))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.status() == reqwest::StatusCode::OK {
        log_for_debugging(&format!(
            "[bridge] Session {} archived successfully",
            session_id
        ));
        Ok(())
    } else {
        let status_code = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        let detail = extract_error_detail_from_text(&body);
        Err(format!(
            "Session archive failed with status {}{}",
            status_code,
            detail.map(|d| format!(": {}", d)).unwrap_or_default()
        ))
    }
}

/// Update the title of a bridge session via PATCH /v1/sessions/{id}.
///
/// Called when the user renames a session via /rename while a bridge
/// connection is active, so the title stays in sync on claude.ai/code.
///
/// Errors are swallowed — title sync is best-effort.
pub async fn update_bridge_session_title(
    session_id: &str,
    title: &str,
    base_url: Option<&str>,
    get_access_token: Option<&dyn Fn() -> Option<String>>,
) {
    // Get access token
    let access_token = get_access_token
        .and_then(|f| f())
        .or_else(|| crate::bridge::get_bridge_access_token());

    let access_token = match access_token {
        Some(t) => t,
        None => {
            log_for_debugging("[bridge] No access token for session title update");
            return;
        }
    };

    // Get organization UUID
    let org_uuid = get_organization_uuid();
    let org_uuid = match org_uuid {
        Some(uuid) => uuid,
        None => {
            log_for_debugging("[bridge] No org UUID for session title update");
            return;
        }
    };

    let headers = build_oauth_headers(&access_token, &org_uuid);

    // Compat gateway only accepts session_* (compat/convert.go:27). v2 callers
    // pass raw cse_*; retag here so all callers can pass whatever they hold.
    // Idempotent for v1's session_* and bridgeMain's pre-converted compatSessionId.
    let compat_id = crate::bridge::to_compat_session_id(session_id);

    let url = format!(
        "{}/v1/sessions/{}",
        base_url.unwrap_or(&get_oauth_config()),
        compat_id
    );

    log_for_debugging(&format!(
        "[bridge] Updating session title: {} → {}",
        compat_id, title
    ));

    let client = reqwest::Client::new();
    match client
        .patch(&url)
        .headers(headers)
        .timeout(std::time::Duration::from_secs(10))
        .json(&serde_json::json!({ "title": title }))
        .send()
        .await
    {
        Ok(response) => {
            if response.status() == reqwest::StatusCode::OK {
                log_for_debugging("[bridge] Session title updated successfully");
            } else {
                let status_code = response.status().as_u16();
                let body = response.text().await.unwrap_or_default();
                let detail = extract_error_detail_from_text(&body);
                log_for_debugging(&format!(
                    "[bridge] Session title update failed with status {}{}",
                    status_code,
                    detail.map(|d| format!(": {}", d)).unwrap_or_default()
                ));
            }
        }
        Err(e) => {
            log_for_debugging(&format!(
                "[bridge] Session title update request failed: {}",
                e
            ));
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Build git source and outcome context from repo URL
fn build_git_context(
    git_repo_url: &str,
    branch: Option<&str>,
) -> (Option<GitSource>, Option<GitOutcome>) {
    // Try to parse git remote URL
    let parsed = parse_git_remote(git_repo_url);

    if let Some((host, owner, name)) = parsed {
        let revision = branch.map(String::from).or_else(get_default_branch);
        let source = GitSource {
            source_type: "git_repository".to_string(),
            url: format!("https://{}/{}/{}", host, owner, name),
            revision,
        };
        let outcome = GitOutcome {
            outcome_type: "git_repository".to_string(),
            git_info: GitInfo {
                info_type: "github".to_string(),
                repo: format!("{}/{}", owner, name),
                branches: vec![format!("claude/{}", branch.unwrap_or("task"))],
            },
        };
        (Some(source), Some(outcome))
    } else {
        // Fallback: try parseGitHubRepository for owner/repo format
        if let Some((owner, name)) = parse_github_repository(git_repo_url) {
            let revision = branch.map(String::from).or_else(get_default_branch);
            let source = GitSource {
                source_type: "git_repository".to_string(),
                url: format!("https://github.com/{}/{}", owner, name),
                revision,
            };
            let outcome = GitOutcome {
                outcome_type: "git_repository".to_string(),
                git_info: GitInfo {
                    info_type: "github".to_string(),
                    repo: format!("{}/{}", owner, name),
                    branches: vec![format!("claude/{}", branch.unwrap_or("task"))],
                },
            };
            (Some(source), Some(outcome))
        } else {
            (None, None)
        }
    }
}

/// Parse git remote URL to extract host, owner, and name
fn parse_git_remote(url: &str) -> Option<(String, String, String)> {
    // Simple HTTPS URL parsing
    // Format: https://host/owner/name or https://host/owner/name.git
    let url = url.trim_end_matches(".git");

    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 3 {
        // Could be host/owner/name or protocol/host/owner/name
        let start = if parts[0] == "https:" || parts[0] == "http:" {
            2
        } else {
            0
        };
        if parts.len() >= start + 3 {
            let host = if start == 2 {
                parts[1].to_string()
            } else {
                "github.com".to_string()
            };
            let owner = parts[start].to_string();
            let name = parts[start + 1].to_string();
            return Some((host, owner, name));
        }
    }
    None
}

/// Parse GitHub repository in owner/repo format
fn parse_github_repository(s: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// Get default branch name (simplified)
fn get_default_branch() -> Option<String> {
    use std::process::Command;
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Get main loop model
fn get_main_loop_model() -> Option<String> {
    // Simplified: would need to get from config
    None
}

/// Get organization UUID
fn get_organization_uuid() -> Option<String> {
    // Simplified: would need to get from OAuth client
    None
}

/// OAuth config
struct OAuthConfig {
    BASE_API_URL: String,
}

fn get_oauth_config() -> String {
    std::env::var(ai::API_BASE_URL).unwrap_or_else(|_| "https://api.claude.ai".to_string())
}

/// Build OAuth headers
fn build_oauth_headers(access_token: &str, org_uuid: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", access_token)) {
        headers.insert(AUTHORIZATION, val);
    }
    headers.insert(
        HeaderName::from_static("anthropic-beta"),
        HeaderValue::from_static("ccr-byoc-2025-07-29"),
    );
    if let Ok(val) = HeaderValue::from_str(org_uuid) {
        headers.insert(HeaderName::from_static("x-organization-uuid"), val);
    }
    // Add Content-Type for JSON
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
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
