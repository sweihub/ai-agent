// Source: /data/home/swei/claudecode/openclaudecode/src/services/api/sessionIngress.ts
//! Session ingress module
//! Handles session log persistence and retrieval

use std::collections::HashMap;

use crate::utils::http::get_user_agent;
use std::sync::Mutex;

use once_cell::sync::Lazy;

/// Max retries for append operations
const MAX_RETRIES: u32 = 10;
/// Base delay for exponential backoff
const BASE_DELAY_MS: u64 = 500;

/// Last UUID map per session
static LAST_UUID_MAP: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Transcript message entry
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TranscriptMessage {
    pub uuid: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Log entry
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Entry {
    pub uuid: Option<String>,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Session ingress error
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionIngressError {
    pub error: Option<SessionIngressErrorDetail>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionIngressErrorDetail {
    pub message: Option<String>,
    pub r#type: Option<String>,
}

/// Teleport events response
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TeleportEventsResponse {
    pub data: Vec<TeleportEvent>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TeleportEvent {
    pub event_id: String,
    pub event_type: String,
    pub is_compaction: bool,
    pub payload: Option<Entry>,
    pub created_at: String,
}

/// Get session ingress auth token
fn get_session_ingress_auth_token() -> Option<String> {
    // TODO: Integrate with auth system
    None
}

/// Get OAuth config
fn get_oauth_config() -> OauthConfig {
    OauthConfig {
        base_api_url: std::env::var("AI_CODE_API_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct OauthConfig {
    pub base_api_url: String,
}

/// Get OAuth headers
fn get_oauth_headers(access_token: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", access_token),
    );
    headers.insert("User-Agent".to_string(), get_user_agent());
    headers
}

/// Log for debugging
fn log_for_debugging(msg: &str) {
    log::debug!("[session-ingress] {}", msg);
}

/// Sleep for specified milliseconds
async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

/// Find last UUID in logs
fn find_last_uuid(logs: &[Entry]) -> Option<String> {
    logs.iter().rev().find_map(|e| e.uuid.clone())
}

/// Fetch session logs from URL
async fn fetch_session_logs_from_url(
    session_id: &str,
    url: &str,
    headers: HashMap<String, String>,
) -> Result<Vec<Entry>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(20000))
        .build()
        .map_err(|e| e.to_string())?;

    // Check for after_last_compact flag
    let mut request = client.get(url).headers(
        headers
            .into_iter()
            .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
            .collect(),
    );

    if std::env::var("AI_CODE_AFTER_LAST_COMPACT")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
    {
        request = request.query(&[("after_last_compact", "true")]);
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    if response.status() == reqwest::StatusCode::OK {
        let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        // Validate response structure
        let loglines = data
            .get("loglines")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "Invalid session logs response format".to_string())?;

        let logs: Vec<Entry> =
            serde_json::from_value(serde_json::json!(loglines)).map_err(|e| e.to_string())?;

        log_for_debugging(&format!(
            "Fetched {} session logs for session {}",
            logs.len(),
            session_id
        ));
        return Ok(logs);
    }

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        log_for_debugging(&format!("No existing logs for session {}", session_id));
        return Ok(Vec::new());
    }

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Your session has expired. Please run /login to sign in again.".to_string());
    }

    Err(format!(
        "Failed to fetch session logs: {}",
        response.status()
    ))
}

/// Internal implementation of appendSessionLog with retry logic
async fn append_session_log_impl(
    session_id: &str,
    entry: TranscriptMessage,
    url: &str,
    headers: HashMap<String, String>,
) -> bool {
    for attempt in 1..=MAX_RETRIES {
        let mut request_headers = headers.clone();

        // Add Last-Uuid header if we have one
        if let Some(last_uuid) = LAST_UUID_MAP.lock().unwrap().get(session_id) {
            request_headers.insert("Last-Uuid".to_string(), last_uuid.clone());
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(30000))
            .build()
            .map_err(|e| e.to_string())
            .unwrap(); // Simplified for retry loop

        let response = client
            .put(url)
            .headers(
                request_headers
                    .iter()
                    .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                    .collect(),
            )
            .json(&entry)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::OK
                    || resp.status() == reqwest::StatusCode::CREATED
                {
                    LAST_UUID_MAP
                        .lock()
                        .unwrap()
                        .insert(session_id.to_string(), entry.uuid.clone());
                    log_for_debugging(&format!(
                        "Successfully persisted session log entry for session {}",
                        session_id
                    ));
                    return true;
                }

                if resp.status() == reqwest::StatusCode::CONFLICT {
                    // Handle 409 conflict
                    let server_last_uuid = resp
                        .headers()
                        .get("x-last-uuid")
                        .and_then(|v| v.to_str().ok())
                        .map(String::from);

                    if server_last_uuid.as_deref() == Some(entry.uuid.as_str()) {
                        // Our entry is already on server
                        LAST_UUID_MAP
                            .lock()
                            .unwrap()
                            .insert(session_id.to_string(), entry.uuid.clone());
                        log_for_debugging(&format!(
                            "Session entry {} already present on server, recovering from stale state",
                            entry.uuid
                        ));
                        return true;
                    }

                    // Adopt server's last UUID
                    if let Some(ref server_uuid) = server_last_uuid {
                        LAST_UUID_MAP
                            .lock()
                            .unwrap()
                            .insert(session_id.to_string(), server_uuid.clone());
                        log_for_debugging(&format!(
                            "Session 409: adopting server lastUuid={}, retrying entry {}",
                            server_uuid, entry.uuid
                        ));
                    }
                    // Continue to retry
                    continue;
                }

                if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
                    log_for_debugging("Session token expired or invalid");
                    return false;
                }

                log_for_debugging(&format!("Failed to persist session log: {}", resp.status()));
            }
            Err(e) => {
                log::error!("Error persisting session log: {}", e);
            }
        }

        if attempt == MAX_RETRIES {
            log_for_debugging(&format!(
                "Remote persistence failed after {} attempts",
                MAX_RETRIES
            ));
            return false;
        }

        let delay_ms = std::cmp::min(BASE_DELAY_MS * 2u64.pow(attempt - 1), 8000);
        log_for_debugging(&format!(
            "Remote persistence attempt {}/{} failed, retrying in {}ms...",
            attempt, MAX_RETRIES, delay_ms
        ));
        sleep_ms(delay_ms).await;
    }

    false
}

/// Append a log entry to the session using JWT token
pub async fn append_session_log(session_id: &str, entry: TranscriptMessage, url: &str) -> bool {
    let session_token = match get_session_ingress_auth_token() {
        Some(token) => token,
        None => {
            log_for_debugging("No session token available for session persistence");
            return false;
        }
    };

    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", session_token),
    );
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("User-Agent".to_string(), get_user_agent());

    // For now, just call impl directly (sequential wrapper would be added in full impl)
    append_session_log_impl(session_id, entry, url, headers).await
}

/// Get all session logs for hydration
pub async fn get_session_logs(session_id: &str, url: &str) -> Result<Vec<Entry>, String> {
    let session_token = match get_session_ingress_auth_token() {
        Some(token) => token,
        None => {
            log_for_debugging("No session token available for fetching session logs");
            return Err("No session token available".to_string());
        }
    };

    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", session_token),
    );
    headers.insert("User-Agent".to_string(), get_user_agent());

    let logs = fetch_session_logs_from_url(session_id, url, headers).await?;

    if let Some(last_entry) = logs.last() {
        if let Some(ref uuid) = last_entry.uuid {
            LAST_UUID_MAP
                .lock()
                .unwrap()
                .insert(session_id.to_string(), uuid.clone());
        }
    }

    Ok(logs)
}

/// Get all session logs for hydration via OAuth
pub async fn get_session_logs_via_oauth(
    session_id: &str,
    access_token: &str,
    org_uuid: &str,
) -> Result<Vec<Entry>, String> {
    let config = get_oauth_config();
    let url = format!(
        "{}/v1/session_ingress/session/{}",
        config.base_api_url, session_id
    );

    log_for_debugging(&format!("Fetching session logs from: {}", url));

    let mut headers = get_oauth_headers(access_token);
    headers.insert("x-organization-uuid".to_string(), org_uuid.to_string());

    fetch_session_logs_from_url(session_id, &url, headers).await
}

/// Get worker events (transcript) via the CCR v2 Sessions API
pub async fn get_teleport_events(
    session_id: &str,
    access_token: &str,
    org_uuid: &str,
) -> Result<Vec<Entry>, String> {
    let config = get_oauth_config();
    let base_url = format!(
        "{}/v1/code/sessions/{}/teleport-events",
        config.base_api_url, session_id
    );

    log_for_debugging(&format!("[teleport] Fetching events from: {}", base_url));

    let mut headers = get_oauth_headers(access_token);
    headers.insert("x-organization-uuid".to_string(), org_uuid.to_string());

    let mut all: Vec<Entry> = Vec::new();
    let mut cursor: Option<String> = None;
    let mut pages = 0;
    let max_pages = 100;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(20000))
        .build()
        .map_err(|e| e.to_string())?;

    while pages < max_pages {
        let mut request = client
            .get(&base_url)
            .headers(
                headers
                    .iter()
                    .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                    .collect(),
            )
            .query(&[("limit", "1000")]);

        if let Some(ref c) = cursor {
            request = request.query(&[("cursor", c.as_str())]);
        }

        let response = request.send().await.map_err(|e| e.to_string())?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            if pages == 0 {
                return Ok(Vec::new());
            }
            return Ok(all);
        }

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(
                "Your session has expired. Please run /login to sign in again.".to_string(),
            );
        }

        if !response.status().is_success() {
            let status = response.status();
            let data: serde_json::Value = response.json().await.unwrap_or_default();
            return Err(format!("Teleport events returned {}: {}", status, data));
        }

        let data: TeleportEventsResponse = response.json().await.map_err(|e| e.to_string())?;

        for ev in data.data {
            if let Some(payload) = ev.payload {
                all.push(payload);
            }
        }

        pages += 1;

        if data.next_cursor.is_none() {
            break;
        }
        cursor = data.next_cursor;
    }

    if pages >= max_pages {
        log::error!(
            "Teleport events hit page cap ({}) for {}",
            max_pages,
            session_id
        );
    }

    log_for_debugging(&format!(
        "[teleport] Fetched {} events over {} page(s) for {}",
        all.len(),
        pages,
        session_id
    ));

    Ok(all)
}

/// Clear cached state for a session
pub fn clear_session(session_id: &str) {
    LAST_UUID_MAP.lock().unwrap().remove(session_id);
}

/// Clear all cached session state
pub fn clear_all_sessions() {
    LAST_UUID_MAP.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_last_uuid() {
        let logs = vec![
            Entry {
                uuid: Some("uuid1".to_string()),
                data: serde_json::json!({}),
            },
            Entry {
                uuid: None,
                data: serde_json::json!({}),
            },
            Entry {
                uuid: Some("uuid3".to_string()),
                data: serde_json::json!({}),
            },
        ];
        let result = find_last_uuid(&logs);
        assert_eq!(result, Some("uuid3".to_string()));
    }

    #[test]
    fn test_find_last_uuid_empty() {
        let logs: Vec<Entry> = vec![];
        let result = find_last_uuid(&logs);
        assert_eq!(result, None);
    }
}
