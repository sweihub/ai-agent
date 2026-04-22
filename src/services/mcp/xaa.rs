// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/xaa.ts
//! Cross-App Access (XAA) / Enterprise Managed Authorization (SEP-990)
//!
//! Obtains an MCP access token WITHOUT a browser consent screen by chaining:
//!   1. RFC 8693 Token Exchange at the IdP: id_token -> ID-JAG
//!   2. RFC 7523 JWT Bearer Grant at the AS: ID-JAG -> access_token

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::utils::http::get_user_agent;

const XAA_REQUEST_TIMEOUT_MS: u64 = 30_000;

/// Token exchange grant type (RFC 8693)
const TOKEN_EXCHANGE_GRANT: &str = "urn:ietf:params:oauth:grant-type:token-exchange";
/// JWT bearer grant type (RFC 7523)
const JWT_BEARER_GRANT: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";
/// ID-JAG token type
const ID_JAG_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:id-jag";
/// ID token type
const ID_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:id_token";
/// Access token type
const ACCESS_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:access_token";

/// XAA token exchange error
#[derive(Debug)]
pub struct XaaTokenExchangeError {
    pub message: String,
    /// Whether to clear the id_token from cache
    pub should_clear_id_token: bool,
}

impl std::fmt::Display for XaaTokenExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for XaaTokenExchangeError {}

impl XaaTokenExchangeError {
    pub fn new(message: String, should_clear_id_token: bool) -> Self {
        Self {
            message,
            should_clear_id_token,
        }
    }

    pub fn with_id_token_clear(mut self) -> Self {
        self.should_clear_id_token = true;
        self
    }
}

/// XAA token exchange request (RFC 8693)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XaaTokenRequest {
    pub grant_type: String,
    pub resource: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_token_type: Option<String>,
}

/// XAA token exchange response (RFC 8693)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XaaTokenResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

/// XAA configuration
#[derive(Debug, Clone)]
pub struct XaaConfig {
    /// Authorization server URL
    pub auth_server_url: String,
    /// MCP server resource URL
    pub resource: String,
    /// Client ID for XAA
    pub client_id: String,
}

/// Normalize URL per RFC 3986 section 6.2.2
pub fn normalize_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let normalized = parsed.to_string();
        normalized.trim_end_matches('/').to_string()
    } else {
        url.trim_end_matches('/').to_string()
    }
}

/// Build an HTTP client for XAA requests
fn build_xaa_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_millis(XAA_REQUEST_TIMEOUT_MS))
        .user_agent(get_user_agent())
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Perform token exchange (RFC 8693)
/// Exchanges an id_token for an ID-JAG at the IdP
pub async fn exchange_token(
    id_token: &str,
    config: &XaaConfig,
) -> Result<String, XaaTokenExchangeError> {
    let client = build_xaa_client().map_err(|e| {
        XaaTokenExchangeError::new(format!("Failed to build HTTP client: {}", e), false)
    })?;

    // Build the token exchange request per RFC 8693
    let form_params = [
        ("grant_type", TOKEN_EXCHANGE_GRANT),
        ("resource", &config.resource),
        ("subject_token", id_token),
        ("subject_token_type", ID_TOKEN_TYPE),
        ("requested_token_type", ID_JAG_TOKEN_TYPE),
    ];

    // The token endpoint is typically at /oauth/token or /token
    let token_url = format!("{}/oauth/token", normalize_url(&config.auth_server_url));

    log::debug!("Performing token exchange at: {}", token_url);

    let response = client
        .post(&token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&form_params)
        .send()
        .await
        .map_err(|e| {
            XaaTokenExchangeError::new(
                format!("Token exchange request failed: {}", e),
                should_clear_on_network_error(&e),
            )
        })?;

    let status = response.status();
    let body = response.text().await.map_err(|e| {
        XaaTokenExchangeError::new(
            format!("Failed to read token exchange response: {}", e),
            false,
        )
    })?;

    if !status.is_success() {
        log::debug!("Token exchange failed with status {}: {}", status, body);

        // Try to parse error response
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&body) {
            let error_desc = error_json
                .get("error_description")
                .or_else(|| error_json.get("error"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");

            let should_clear = error_desc.contains("invalid_grant")
                || error_desc.contains("invalid_token")
                || error_desc.contains("expired");

            return Err(XaaTokenExchangeError::new(
                format!("Token exchange failed ({}): {}", status, error_desc),
                should_clear,
            ));
        }

        return Err(XaaTokenExchangeError::new(
            format!("Token exchange failed with status {}: {}", status, body),
            status.is_client_error(),
        ));
    }

    // Parse the token response
    let token_response = serde_json::from_str::<XaaTokenResponse>(&body).map_err(|e| {
        XaaTokenExchangeError::new(
            format!("Failed to parse token exchange response: {}", e),
            false,
        )
    })?;

    // The exchanged token is in access_token field per RFC 8693
    Ok(token_response.access_token)
}

/// Request JWT bearer grant (RFC 7523)
/// Exchanges ID-JAG for access_token at the authorization server
pub async fn request_jwt_bearer_grant(
    id_jag: &str,
    config: &XaaConfig,
) -> Result<XaaTokenResponse, String> {
    let client = build_xaa_client()?;

    // Build the JWT bearer grant request per RFC 7523
    let form_params = [
        ("grant_type", JWT_BEARER_GRANT),
        ("resource", &config.resource),
        ("assertion", id_jag),
    ];

    let token_url = format!("{}/oauth/token", normalize_url(&config.auth_server_url));

    log::debug!("Performing JWT bearer grant at: {}", token_url);

    let response = client
        .post(&token_url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&form_params)
        .send()
        .await
        .map_err(|e| format!("JWT bearer grant request failed: {}", e))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read JWT bearer response: {}", e))?;

    if !status.is_success() {
        log::debug!("JWT bearer grant failed with status {}: {}", status, body);

        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&body) {
            let error_desc = error_json
                .get("error_description")
                .or_else(|| error_json.get("error"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(format!(
                "JWT bearer grant failed ({}): {}",
                status, error_desc
            ));
        }

        return Err(format!(
            "JWT bearer grant failed with status {}: {}",
            status, body
        ));
    }

    serde_json::from_str::<XaaTokenResponse>(&body)
        .map_err(|e| format!("Failed to parse JWT bearer response: {}", e))
}

/// Perform full XAA flow: id_token -> access_token
/// This chains the token exchange and JWT bearer grant
pub async fn get_xaa_access_token(
    id_token: &str,
    config: &XaaConfig,
) -> Result<XaaTokenResponse, XaaTokenExchangeError> {
    log::debug!("Starting XAA flow for resource: {}", config.resource);

    // Step 1: Exchange id_token for ID-JAG
    let id_jag = exchange_token(id_token, config).await?;
    log::debug!("Received ID-JAG from token exchange");

    // Step 2: Exchange ID-JAG for access_token
    let response = request_jwt_bearer_grant(&id_jag, config)
        .await
        .map_err(|e| XaaTokenExchangeError::new(e, false))?;

    log::debug!(
        "XAA flow completed. Token type: {:?}, Expires in: {:?}",
        response.issued_token_type,
        response.expires_in
    );

    Ok(response)
}

/// XAA cross-app access token request
#[derive(Debug, Clone)]
pub struct CrossAppAccessRequest {
    pub id_token: String,
    pub auth_server_url: String,
    pub resource: String,
    pub client_id: String,
}

/// Perform cross-app access token exchange
pub async fn perform_cross_app_access(
    request: CrossAppAccessRequest,
) -> Result<String, XaaTokenExchangeError> {
    let config = XaaConfig {
        auth_server_url: request.auth_server_url,
        resource: request.resource,
        client_id: request.client_id,
    };

    let response = get_xaa_access_token(&request.id_token, &config).await?;
    Ok(response.access_token)
}

/// Determine if we should clear the cached id_token based on the error
fn should_clear_on_network_error(_error: &reqwest::Error) -> bool {
    // Network errors are typically transient - don't clear the token
    // Only clear on auth-specific errors (invalid_token, expired, etc.)
    false
}

/// Validate that an id_token looks well-formed (basic check)
pub fn is_valid_id_token(token: &str) -> bool {
    // JWT tokens have 3 parts separated by dots
    let parts: Vec<&str> = token.split('.').collect();
    parts.len() == 3 && !token.is_empty() && token.len() > 20
}

/// Check if XAA is enabled via environment variable
pub fn is_xaa_enabled() -> bool {
    std::env::var("AI_CODE_ENABLE_XAA")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        // normalize_url trims trailing slashes
        assert_eq!(normalize_url("https://example.com/"), "https://example.com");
        assert_eq!(normalize_url("https://example.com"), "https://example.com");
        assert_eq!(
            normalize_url("https://example.com/path/"),
            "https://example.com/path"
        );
        assert_eq!(
            normalize_url("https://api.example.com/v1/"),
            "https://api.example.com/v1"
        );
    }

    #[test]
    fn test_is_valid_id_token() {
        // Valid JWT format
        assert!(is_valid_id_token(
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0In0.testsignature"
        ));
        // Invalid formats
        assert!(!is_valid_id_token("not-a-jwt"));
        assert!(!is_valid_id_token("only.two"));
        assert!(!is_valid_id_token(""));
        assert!(!is_valid_id_token("a.b.c")); // Too short
    }

    #[test]
    fn test_is_xaa_enabled_default() {
        assert!(!is_xaa_enabled());
    }

    #[test]
    fn test_xaa_token_request_serialization() {
        let request = XaaTokenRequest {
            grant_type: TOKEN_EXCHANGE_GRANT.to_string(),
            resource: "https://api.example.com".to_string(),
            requested_token_type: Some(ID_JAG_TOKEN_TYPE.to_string()),
            subject_token: Some("test-token".to_string()),
            subject_token_type: Some(ID_TOKEN_TYPE.to_string()),
            actor_token: None,
            actor_token_type: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("token-exchange"));
        assert!(json.contains("resource"));
    }

    #[test]
    fn test_xaa_token_response_deserialization() {
        let json = r#"{
            "accessToken": "test-access-token",
            "issuedTokenType": "urn:ietf:params:oauth:token-type:access_token",
            "tokenType": "Bearer",
            "expiresIn": 3600,
            "scope": "read write"
        }"#;

        let response: XaaTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "test-access-token");
        assert_eq!(response.token_type, Some("Bearer".to_string()));
        assert_eq!(response.expires_in, Some(3600));
    }

    #[test]
    fn test_xaa_error_display() {
        let error = XaaTokenExchangeError::new("Test error".to_string(), false);
        assert_eq!(format!("{}", error), "Test error");
    }

    #[test]
    fn test_xaa_error_with_clear() {
        let error =
            XaaTokenExchangeError::new("Test error".to_string(), false).with_id_token_clear();
        assert!(error.should_clear_id_token);
    }

    #[test]
    fn test_xaa_config() {
        let config = XaaConfig {
            auth_server_url: "https://auth.example.com".to_string(),
            resource: "https://api.example.com".to_string(),
            client_id: "test-client".to_string(),
        };
        assert_eq!(config.auth_server_url, "https://auth.example.com");
        assert_eq!(config.resource, "https://api.example.com");
    }
}
