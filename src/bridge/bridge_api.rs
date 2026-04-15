//! Bridge API client implementation.
//!
//! Translated from openclaudecode/src/bridge/bridgeApi.ts

use serde::{Deserialize, Serialize};

// =============================================================================
// CONSTANTS
// =============================================================================

const BETA_HEADER: &str = "environments-2025-11-01";
const EMPTY_POLL_LOG_INTERVAL: usize = 100;

// Safe pattern for server-provided IDs in URL paths
const SAFE_ID_PATTERN: &str = r"^[a-zA-Z0-9_-]+$";

// =============================================================================
// TYPES
// =============================================================================

/// Work data from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkData {
    #[serde(rename = "type")]
    pub data_type: String,
    pub id: String,
}

/// Work response from poll endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub environment_id: String,
    pub state: String,
    pub data: WorkData,
    pub secret: String, // base64url-encoded JSON
    pub created_at: String,
}

/// Permission response event sent to a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponseEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub response: PermissionResponseInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponseInner {
    #[serde(rename = "subtype")]
    pub response_subtype: String,
    pub request_id: String,
    pub response: serde_json::Value,
}

/// Bridge configuration for environment registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub machine_name: String,
    pub dir: String,
    pub branch: String,
    #[serde(rename = "gitRepoUrl")]
    pub git_repo_url: Option<String>,
    #[serde(rename = "maxSessions")]
    pub max_sessions: u32,
    #[serde(rename = "bridgeId")]
    pub bridge_id: String,
    #[serde(rename = "workerType")]
    pub worker_type: String,
    #[serde(rename = "reuseEnvironmentId")]
    pub reuse_environment_id: Option<String>,
    #[serde(rename = "apiBaseUrl")]
    pub api_base_url: String,
}

/// Registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    #[serde(rename = "environment_id")]
    pub environment_id: String,
    #[serde(rename = "environment_secret")]
    pub environment_secret: String,
}

/// Heartbeat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    #[serde(rename = "lease_extended")]
    pub lease_extended: bool,
    pub state: String,
}

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Full error printed when `claude remote-control` is run without auth.
pub const BRIDGE_LOGIN_ERROR: &str = "Error: You must be logged in to use Remote Control.\n\n\
    Remote Control is only available with claude.ai subscriptions. Please use `/login` to \
    sign in with your claude.ai account.";

/// Reusable login guidance appended to bridge auth errors.
pub const BRIDGE_LOGIN_INSTRUCTION: &str = "Remote Control is only available with claude.ai \
    subscriptions. Please use `/login` to sign in with your claude.ai account.";

/// Fatal bridge errors that should not be retried
#[derive(Debug)]
pub struct BridgeFatalError {
    pub message: String,
    pub status: u16,
    pub error_type: Option<String>,
}

impl std::fmt::Display for BridgeFatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BridgeFatalError {}

impl BridgeFatalError {
    pub fn new(message: String, status: u16, error_type: Option<String>) -> Self {
        Self {
            message,
            status,
            error_type,
        }
    }
}

// =============================================================================
// API CLIENT
// =============================================================================

/// Bridge API client dependencies
pub struct BridgeApiDeps {
    pub base_url: String,
    pub get_access_token: Box<dyn Fn() -> Option<String> + Send + Sync>,
    pub runner_version: String,
    pub on_debug: Option<Box<dyn Fn(&str) + Send + Sync>>,
    /// Returns the trusted device token
    pub get_trusted_device_token: Option<Box<dyn Fn() -> Option<String> + Send + Sync>>,
}

impl Default for BridgeApiDeps {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            get_access_token: Box::new(|| None),
            runner_version: String::new(),
            on_debug: None,
            get_trusted_device_token: None,
        }
    }
}

// Note: BridgeApiClient trait removed in favor of SyncBridgeApiClient struct

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Validate that a server-provided ID is safe to interpolate into a URL path.
pub fn validate_bridge_id(id: &str, label: &str) -> Result<String, String> {
    if id.is_empty() || !Regex::new(SAFE_ID_PATTERN).unwrap().is_match(id) {
        return Err(format!("Invalid {}: contains unsafe characters", label));
    }
    Ok(id.to_string())
}

/// Check whether an error type string indicates a session/environment expiry.
pub fn is_expired_error_type(error_type: Option<&str>) -> bool {
    match error_type {
        Some(etype) => etype.contains("expired") || etype.contains("lifetime"),
        None => false,
    }
}

/// Check whether a BridgeFatalError is a suppressible 403 permission error.
pub fn is_suppressible_403(err: &BridgeFatalError) -> bool {
    if err.status != 403 {
        return false;
    }
    err.message.contains("external_poll_sessions") || err.message.contains("environments:manage")
}

fn extract_error_type_from_data(data: &serde_json::Value) -> Option<String> {
    if let Some(error) = data.get("error").and_then(|v| v.as_object()) {
        if let Some(etype) = error.get("type").and_then(|v| v.as_str()) {
            return Some(etype.to_string());
        }
    }
    None
}

fn extract_error_detail(data: &serde_json::Value) -> Option<String> {
    if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
        return Some(msg.to_string());
    }
    if let Some(error) = data.get("error").and_then(|v| v.as_object()) {
        if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
    }
    None
}

// =============================================================================
// SIMPLE SYNC IMPLEMENTATION (no async)
// =============================================================================

/// Synchronous bridge API client for simple use cases
pub struct SyncBridgeApiClient {
    base_url: String,
    get_access_token: Box<dyn Fn() -> Option<String> + Send + Sync>,
    runner_version: String,
    get_trusted_device_token: Option<Box<dyn Fn() -> Option<String> + Send + Sync>>,
    on_debug: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl SyncBridgeApiClient {
    pub fn new(
        base_url: String,
        get_access_token: impl Fn() -> Option<String> + Send + Sync + 'static,
        runner_version: String,
    ) -> Self {
        Self {
            base_url,
            get_access_token: Box::new(get_access_token),
            runner_version,
            get_trusted_device_token: None,
            on_debug: None,
        }
    }

    pub fn with_trusted_device_token(
        mut self,
        getter: impl Fn() -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        self.get_trusted_device_token = Some(Box::new(getter));
        self
    }

    pub fn with_debug(mut self, debug: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_debug = Some(Box::new(debug));
        self
    }

    fn debug(&self, msg: &str) {
        if let Some(ref debug) = self.on_debug {
            debug(msg);
        }
    }

    fn get_headers(&self, access_token: &str) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
        headers.insert("anthropic-beta", BETA_HEADER.parse().unwrap());
        headers.insert(
            "x-environment-runner-version",
            self.runner_version.parse().unwrap(),
        );

        if let Some(ref getter) = self.get_trusted_device_token {
            if let Some(token) = getter() {
                headers.insert("X-Trusted-Device-Token", token.parse().unwrap());
            }
        }

        headers
    }

    fn resolve_auth(&self) -> Result<String, BridgeFatalError> {
        match (self.get_access_token)() {
            Some(token) => Ok(token),
            None => Err(BridgeFatalError::new(
                BRIDGE_LOGIN_INSTRUCTION.to_string(),
                401,
                None,
            )),
        }
    }

    /// Register this bridge environment
    pub fn register_bridge_environment(
        &self,
        config: BridgeConfig,
    ) -> Result<RegistrationResponse, String> {
        validate_bridge_id(&config.bridge_id, "bridgeId").map_err(|e| e.to_string())?;

        self.debug(&format!(
            "[bridge:api] POST /v1/environments/bridge bridgeId={}",
            config.bridge_id
        ));

        let client = reqwest::blocking::Client::new();
        let token = self.resolve_auth().map_err(|e| e.to_string())?;

        let mut body = serde_json::json!({
            "machine_name": config.machine_name,
            "directory": config.dir,
            "branch": config.branch,
            "git_repo_url": config.git_repo_url,
            "max_sessions": config.max_sessions,
            "metadata": { "worker_type": config.worker_type },
        });

        if let Some(reuse_id) = config.reuse_environment_id {
            body["environment_id"] = serde_json::json!(reuse_id);
        }

        let response = client
            .post(&format!("{}/v1/environments/bridge", self.base_url))
            .headers(self.get_headers(&token))
            .json(&body)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or_default();

        if status != 200 && status != 201 {
            return Err(handle_error_status_sync(status, &data, "Registration"));
        }

        let result: RegistrationResponse = serde_json::from_value(data.clone())
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        self.debug(&format!(
            "[bridge:api] POST /v1/environments/bridge -> {} environment_id={}",
            status, result.environment_id
        ));

        Ok(result)
    }

    /// Poll for work from the environment
    pub fn poll_for_work(
        &self,
        environment_id: &str,
        environment_secret: &str,
        reclaim_older_than_ms: Option<u64>,
    ) -> Result<Option<WorkResponse>, String> {
        validate_bridge_id(environment_id, "environmentId")?;

        let client = reqwest::blocking::Client::new();

        let mut url = format!(
            "{}/v1/environments/{}/work/poll",
            self.base_url, environment_id
        );
        if let Some(ms) = reclaim_older_than_ms {
            url = format!("{}?reclaim_older_than_ms={}", url, ms);
        }

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", environment_secret))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or(serde_json::Value::Null);

        if status != 200 && status != 204 {
            return Err(handle_error_status_sync(status, &data, "Poll"));
        }

        if data.is_null() || data.is_array() {
            return Ok(None);
        }

        let work: WorkResponse = serde_json::from_value(data.clone())
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        self.debug(&format!(
            "[bridge:api] GET .../work/poll -> {} workId={} type={}",
            status, work.id, work.data.data_type
        ));

        Ok(Some(work))
    }

    /// Acknowledge work receipt
    pub fn acknowledge_work(
        &self,
        environment_id: &str,
        work_id: &str,
        session_token: &str,
    ) -> Result<(), String> {
        validate_bridge_id(environment_id, "environmentId")?;
        validate_bridge_id(work_id, "workId")?;

        self.debug(&format!("[bridge:api] POST .../work/{}/ack", work_id));

        let client = reqwest::blocking::Client::new();

        let response = client
            .post(&format!(
                "{}/v1/environments/{}/work/{}/ack",
                self.base_url, environment_id, work_id
            ))
            .headers(self.get_headers(session_token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or_default();

        if status != 200 && status != 204 {
            return Err(handle_error_status_sync(status, &data, "Acknowledge"));
        }

        Ok(())
    }

    /// Stop a work item
    pub fn stop_work(
        &self,
        environment_id: &str,
        work_id: &str,
        force: bool,
    ) -> Result<(), String> {
        validate_bridge_id(environment_id, "environmentId")?;
        validate_bridge_id(work_id, "workId")?;

        self.debug(&format!(
            "[bridge:api] POST .../work/{}/stop force={}",
            work_id, force
        ));

        let client = reqwest::blocking::Client::new();
        let token = self.resolve_auth().map_err(|e| e.to_string())?;

        let response = client
            .post(&format!(
                "{}/v1/environments/{}/work/{}/stop",
                self.base_url, environment_id, work_id
            ))
            .headers(self.get_headers(&token))
            .json(&serde_json::json!({ "force": force }))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or_default();

        if status != 200 && status != 204 {
            return Err(handle_error_status_sync(status, &data, "StopWork"));
        }

        Ok(())
    }

    /// Deregister the environment
    pub fn deregister_environment(&self, environment_id: &str) -> Result<(), String> {
        validate_bridge_id(environment_id, "environmentId")?;

        self.debug(&format!(
            "[bridge:api] DELETE /v1/environments/bridge/{}",
            environment_id
        ));

        let client = reqwest::blocking::Client::new();
        let token = self.resolve_auth().map_err(|e| e.to_string())?;

        let response = client
            .delete(&format!(
                "{}/v1/environments/bridge/{}",
                self.base_url, environment_id
            ))
            .headers(self.get_headers(&token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or_default();

        if status != 200 && status != 204 {
            return Err(handle_error_status_sync(status, &data, "Deregister"));
        }

        Ok(())
    }

    /// Archive a session
    pub fn archive_session(&self, session_id: &str) -> Result<(), String> {
        validate_bridge_id(session_id, "sessionId")?;

        self.debug(&format!(
            "[bridge:api] POST /v1/sessions/{}/archive",
            session_id
        ));

        let client = reqwest::blocking::Client::new();
        let token = self.resolve_auth().map_err(|e| e.to_string())?;

        let response = client
            .post(&format!(
                "{}/v1/sessions/{}/archive",
                self.base_url, session_id
            ))
            .headers(self.get_headers(&token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or_default();

        // 409 = already archived (idempotent, not an error)
        if status == 409 {
            self.debug(&format!(
                "[bridge:api] POST /v1/sessions/{}/archive -> 409 (already archived)",
                session_id
            ));
            return Ok(());
        }

        if status != 200 && status != 204 {
            return Err(handle_error_status_sync(status, &data, "ArchiveSession"));
        }

        Ok(())
    }

    /// Reconnect a session
    pub fn reconnect_session(&self, environment_id: &str, session_id: &str) -> Result<(), String> {
        validate_bridge_id(environment_id, "environmentId")?;
        validate_bridge_id(session_id, "sessionId")?;

        self.debug(&format!(
            "[bridge:api] POST /v1/environments/{}/bridge/reconnect session_id={}",
            environment_id, session_id
        ));

        let client = reqwest::blocking::Client::new();
        let token = self.resolve_auth().map_err(|e| e.to_string())?;

        let response = client
            .post(&format!(
                "{}/v1/environments/{}/bridge/reconnect",
                self.base_url, environment_id
            ))
            .headers(self.get_headers(&token))
            .json(&serde_json::json!({ "session_id": session_id }))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or_default();

        if status != 200 && status != 204 {
            return Err(handle_error_status_sync(status, &data, "ReconnectSession"));
        }

        Ok(())
    }

    /// Send heartbeat for a work item
    pub fn heartbeat_work(
        &self,
        environment_id: &str,
        work_id: &str,
        session_token: &str,
    ) -> Result<HeartbeatResponse, String> {
        validate_bridge_id(environment_id, "environmentId")?;
        validate_bridge_id(work_id, "workId")?;

        self.debug(&format!("[bridge:api] POST .../work/{}/heartbeat", work_id));

        let client = reqwest::blocking::Client::new();

        let response = client
            .post(&format!(
                "{}/v1/environments/{}/work/{}/heartbeat",
                self.base_url, environment_id, work_id
            ))
            .headers(self.get_headers(session_token))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or_default();

        if status != 200 && status != 204 {
            return Err(handle_error_status_sync(status, &data, "Heartbeat"));
        }

        let result: HeartbeatResponse = serde_json::from_value(data.clone())
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        self.debug(&format!(
            "[bridge:api] POST .../work/{}/heartbeat -> {} lease_extended={} state={}",
            work_id, status, result.lease_extended, result.state
        ));

        Ok(result)
    }

    /// Send permission response event
    pub fn send_permission_response_event(
        &self,
        session_id: &str,
        event: PermissionResponseEvent,
        session_token: &str,
    ) -> Result<(), String> {
        validate_bridge_id(session_id, "sessionId")?;

        self.debug(&format!(
            "[bridge:api] POST /v1/sessions/{}/events type={}",
            session_id, event.event_type
        ));

        let client = reqwest::blocking::Client::new();

        let response = client
            .post(&format!(
                "{}/v1/sessions/{}/events",
                self.base_url, session_id
            ))
            .headers(self.get_headers(session_token))
            .json(&serde_json::json!({ "events": [event] }))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let data: serde_json::Value = response.json().unwrap_or_default();

        if status != 200 && status != 204 {
            return Err(handle_error_status_sync(
                status,
                &data,
                "SendPermissionResponseEvent",
            ));
        }

        Ok(())
    }
}

fn handle_error_status_sync(status: u16, data: &serde_json::Value, context: &str) -> String {
    let detail = extract_error_detail(data);
    let error_type = extract_error_type_from_data(data);

    match status {
        401 => format!(
            "{}: Authentication failed (401){}. {}",
            context,
            detail.map(|d| format!(": {}", d)).unwrap_or_default(),
            BRIDGE_LOGIN_INSTRUCTION
        ),
        403 => {
            if is_expired_error_type(error_type.as_deref()) {
                "Remote Control session has expired. Please restart with `claude remote-control` or /remote-control.".to_string()
            } else {
                format!(
                    "{}: Access denied (403){}. Check your organization permissions.",
                    context,
                    detail.map(|d| format!(": {}", d)).unwrap_or_default()
                )
            }
        }
        404 => detail.unwrap_or_else(|| {
            format!(
                "{}: Not found (404). Remote Control may not be available for this organization.",
                context
            )
        }),
        410 => detail.unwrap_or_else(|| {
            "Remote Control session has expired. Please restart with `claude remote-control` or /remote-control.".to_string()
        }),
        429 => format!("{}: Rate limited (429). Polling too frequently.", context),
        _ => format!(
            "{}: Failed with status {}{}",
            context,
            status,
            detail.map(|d| format!(": {}", d)).unwrap_or_default()
        ),
    }
}

// =============================================================================
// REGEX
// =============================================================================

use regex::Regex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bridge_id() {
        assert!(validate_bridge_id("abc123", "test").is_ok());
        assert!(validate_bridge_id("abc-def_123", "test").is_ok());
        assert!(validate_bridge_id("", "test").is_err());
        assert!(validate_bridge_id("../admin", "test").is_err());
        assert!(validate_bridge_id("abc/def", "test").is_err());
    }

    #[test]
    fn test_is_expired_error_type() {
        assert!(is_expired_error_type(Some("session_expired")));
        assert!(is_expired_error_type(Some("environment_lifetime")));
        assert!(!is_expired_error_type(Some("other_error")));
        assert!(!is_expired_error_type(None));
    }

    #[test]
    fn test_is_suppressible_403() {
        let err = BridgeFatalError::new(
            "403: external_poll_sessions not allowed".to_string(),
            403,
            None,
        );
        assert!(is_suppressible_403(&err));

        let err2 = BridgeFatalError::new("403: Some other error".to_string(), 403, None);
        assert!(!is_suppressible_403(&err2));

        let err3 = BridgeFatalError::new("401: Unauthorized".to_string(), 401, None);
        assert!(!is_suppressible_403(&err3));
    }
}
