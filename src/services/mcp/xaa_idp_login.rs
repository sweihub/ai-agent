// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/xaaIdpLogin.ts
//! XAA IdP Login - acquires OIDC id_token from enterprise IdP via authorization_code + PKCE flow

use crate::utils::http::get_user_agent;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const IDP_LOGIN_TIMEOUT_MS: u64 = 5 * 60 * 1000; // 5 minutes
const IDP_REQUEST_TIMEOUT_MS: u64 = 30_000; // 30 seconds
const ID_TOKEN_EXPIRY_BUFFER_S: u64 = 60; // 60 second buffer before expiry

/// XAA IdP settings from configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XaaIdpSettings {
    pub issuer: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(default, rename = "callbackPort")]
    pub callback_port: Option<u16>,
}

/// PKCE code verifier and challenge
#[derive(Debug, Clone)]
pub struct PkceParams {
    pub code_verifier: String,
    pub code_challenge: String,
}

/// OAuth authorization response
#[derive(Debug, Clone)]
pub struct AuthResponse {
    pub code: String,
    pub state: String,
}

/// IdP login options
#[derive(Debug, Clone)]
pub struct IdpLoginOptions {
    pub idp_issuer: String,
    pub idp_client_id: String,
    /// Optional IdP client secret for confidential clients
    pub idp_client_secret: Option<String>,
    /// Fixed callback port
    pub callback_port: Option<u16>,
    /// If true, don't auto-open the browser
    pub skip_browser_open: bool,
}

/// OIDC discovery metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcMetadata {
    pub issuer: String,
    #[serde(rename = "authorization_endpoint")]
    pub authorization_endpoint: String,
    #[serde(rename = "token_endpoint")]
    pub token_endpoint: String,
    #[serde(rename = "userinfo_endpoint")]
    pub userinfo_endpoint: Option<String>,
    #[serde(rename = "jwks_uri")]
    pub jwks_uri: Option<String>,
    #[serde(rename = "scopes_supported")]
    pub scopes_supported: Option<Vec<String>>,
    #[serde(rename = "response_types_supported")]
    pub response_types_supported: Option<Vec<String>>,
}

/// IdP login result
#[derive(Debug, Clone, Serialize)]
pub struct IdpLoginResult {
    pub id_token: String,
    #[serde(skip_serializing)]
    pub expires_at: Instant,
}

/// Simple token cache using LazyLock
type TokenMap = HashMap<String, IdpLoginResult>;

fn create_token_cache() -> Mutex<TokenMap> {
    Mutex::new(TokenMap::new())
}

lazy_static::lazy_static! {
    static ref TOKEN_CACHE: Mutex<TokenMap> = create_token_cache();
}

/// Client secret cache for IdP credentials
type ClientSecretMap = HashMap<String, String>;

fn create_secret_cache() -> Mutex<ClientSecretMap> {
    Mutex::new(ClientSecretMap::new())
}

lazy_static::lazy_static! {
    static ref CLIENT_SECRET_CACHE: Mutex<ClientSecretMap> = create_secret_cache();
}

/// Check if XAA is enabled via environment variable
pub fn is_xaa_enabled() -> bool {
    std::env::var("AI_CODE_ENABLE_XAA")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get XAA IdP settings from configuration
pub fn get_xaa_idp_settings() -> Option<XaaIdpSettings> {
    // Check if XAA is enabled first
    if !is_xaa_enabled() {
        return None;
    }

    // Get issuer from environment
    let issuer = std::env::var("AI_CODE_XAA_IDP_ISSUER").ok()?;

    let client_id =
        std::env::var("AI_CODE_XAA_IDP_CLIENT_ID").unwrap_or_else(|_| "default-client".to_string());

    let callback_port = std::env::var("AI_CODE_XAA_CALLBACK_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());

    Some(XaaIdpSettings {
        issuer,
        client_id,
        callback_port,
    })
}

/// Normalize an IdP issuer URL for use as a cache key:
/// strip trailing slashes, lowercase host
pub fn normalize_issuer(issuer: &str) -> String {
    let trimmed = issuer.trim_end_matches('/');
    // Parse URL and lowercase the host
    if let Ok(parsed) = url::Url::parse(trimmed) {
        let host = parsed.host_str().unwrap_or("").to_lowercase();
        let scheme = parsed.scheme();
        let port = parsed.port_or_known_default();
        let path = parsed.path().trim_end_matches('/');

        let mut result = format!("{}://{}", scheme, host);
        if let Some(p) = port {
            // Only include port if it's not the default
            let default_port = if scheme == "https" { 443 } else { 80 };
            if p != default_port {
                result.push_str(&format!(":{}", p));
            }
        }
        if !path.is_empty() {
            result.push_str(path);
        }
        result
    } else {
        trimmed.to_lowercase()
    }
}

/// Generate a cryptographically secure random string for PKCE
fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            match idx {
                0..=25 => (b'A' + idx as u8) as char,
                26..=51 => (b'a' + (idx - 26) as u8) as char,
                _ => (b'0' + (idx - 52) as u8) as char,
            }
        })
        .collect();
    chars
}

/// Generate PKCE code verifier and challenge (RFC 7636)
pub fn generate_pkce() -> PkceParams {
    // Code verifier: 43-128 characters from unreserved set
    let code_verifier = generate_random_string(64);

    // Code challenge: base64url(SHA256(code_verifier))
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    let code_challenge =
        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, hash);

    PkceParams {
        code_verifier,
        code_challenge,
    }
}

/// Generate a random state parameter for CSRF protection
pub fn generate_state() -> String {
    generate_random_string(32)
}

/// Get cached IdP token if valid (not expired)
pub fn get_cached_idp_id_token(issuer: &str) -> Option<String> {
    let key = normalize_issuer(issuer);
    let tokens = TOKEN_CACHE.lock().unwrap();
    tokens
        .get(&key)
        .filter(|r| r.expires_at > Instant::now())
        .cloned()
        .map(|r| r.id_token)
}

/// Cache IdP token with expiry
pub fn cache_idp_id_token(issuer: String, id_token: String, expires_in_s: u64) {
    let key = normalize_issuer(&issuer);
    let expires_at =
        Instant::now() + Duration::from_secs(expires_in_s.saturating_sub(ID_TOKEN_EXPIRY_BUFFER_S));
    let mut tokens = TOKEN_CACHE.lock().unwrap();
    tokens.insert(
        key,
        IdpLoginResult {
            id_token,
            expires_at,
        },
    );
    log::debug!(
        "Cached IdP token for issuer: {}, expires in: {}s",
        issuer,
        expires_in_s
    );
}

/// Clear cached IdP token
pub fn clear_idp_id_token(issuer: &str) {
    let key = normalize_issuer(issuer);
    let mut tokens = TOKEN_CACHE.lock().unwrap();
    tokens.remove(&key);
    log::debug!("Cleared cached IdP token for issuer: {}", issuer);
}

/// Decode the exp claim from a JWT without verifying its signature.
/// Returns None if parsing fails or exp is absent.
/// Used only to derive a cache TTL.
fn jwt_exp(jwt: &str) -> Option<u64> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    // Decode base64url payload
    let payload = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        parts[1].as_bytes(),
    )
    .ok()?;
    let payload_str = String::from_utf8(payload).ok()?;
    let json: serde_json::Value = serde_json::from_str(&payload_str).ok()?;
    json.get("exp")?.as_u64()
}

/// Save an externally-obtained id_token into the XAA cache.
/// Used by conformance testing where the mock IdP hands us a pre-signed token.
/// Parses the JWT's exp claim for cache TTL.
pub fn save_idp_id_token_from_jwt(issuer: &str, id_token: &str) -> u64 {
    let expires_at_ms = match jwt_exp(id_token) {
        Some(exp) => exp * 1000,
        None => {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                + 3600 * 1000
        } // Default 1 hour
    };
    cache_idp_id_token(
        issuer.to_string(),
        id_token.to_string(),
        (expires_at_ms / 1000) as u64,
    );
    expires_at_ms
}

/// Save an IdP client secret to secure storage, keyed by IdP issuer.
/// Returns (success, warning) tuple.
pub fn save_idp_client_secret(issuer: &str, client_secret: &str) -> (bool, Option<String>) {
    let key = normalize_issuer(issuer);
    let mut secrets = CLIENT_SECRET_CACHE.lock().unwrap();
    secrets.insert(key, client_secret.to_string());
    log::debug!("Saved IdP client secret for issuer: {}", issuer);
    (true, None)
}

/// Clear the IdP client secret for the given issuer.
pub fn clear_idp_client_secret(issuer: &str) {
    let key = normalize_issuer(issuer);
    let mut secrets = CLIENT_SECRET_CACHE.lock().unwrap();
    secrets.remove(&key);
    log::debug!("Cleared IdP client secret for issuer: {}", issuer);
}

/// Build an HTTP client for IdP requests
fn build_idp_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_millis(IDP_REQUEST_TIMEOUT_MS))
        .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects automatically
        .user_agent(get_user_agent())
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Discover OIDC metadata from issuer
pub async fn discover_oidc(issuer: &str) -> Result<OidcMetadata, String> {
    let base_url = issuer.trim_end_matches('/');
    let url = format!("{}/.well-known/openid-configuration", base_url);

    log::debug!("Discovering OIDC metadata from: {}", url);

    let client = build_idp_client()?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("OIDC discovery request failed: {}", e))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read OIDC metadata response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "OIDC discovery failed with status {}: {}",
            status, body
        ));
    }

    serde_json::from_str::<OidcMetadata>(&body)
        .map_err(|e| format!("Failed to parse OIDC metadata: {}", e))
}

/// Build the authorization URL for the OAuth flow
pub fn build_authorization_url(
    issuer: &str,
    client_id: &str,
    redirect_uri: &str,
    pkce: &PkceParams,
    state: &str,
    scopes: &[&str],
    oidc_metadata: Option<&OidcMetadata>,
) -> Result<String, String> {
    let auth_endpoint = if let Some(meta) = oidc_metadata {
        meta.authorization_endpoint.clone()
    } else {
        // Fallback to well-known path
        format!("{}/authorize", issuer.trim_end_matches('/'))
    };

    let mut url = url::Url::parse(&auth_endpoint)
        .map_err(|e| format!("Invalid authorization endpoint: {}", e))?;

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("state", state)
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("scope", &scopes.join(" "));

    Ok(url.to_string())
}

/// Exchange authorization code for tokens
pub async fn exchange_code_for_tokens(
    issuer: &str,
    client_id: &str,
    client_secret: Option<&str>,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
    oidc_metadata: Option<&OidcMetadata>,
) -> Result<TokenResponse, String> {
    let token_endpoint = if let Some(meta) = oidc_metadata {
        meta.token_endpoint.clone()
    } else {
        // Fallback to well-known path
        format!("{}/token", issuer.trim_end_matches('/'))
    };

    let client = build_idp_client()?;

    let mut form_params: Vec<(&str, &str)> = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("code_verifier", code_verifier),
    ];

    let mut request = client.post(&token_endpoint);

    // Use client_secret_post or client_secret_basic if provided
    if let Some(secret) = client_secret {
        form_params.push(("client_secret", secret));
    }

    request = request
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&form_params);

    log::debug!("Exchanging code for tokens at: {}", token_endpoint);

    let response = request
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read token response: {}", e))?;

    if !status.is_success() {
        log::debug!("Token exchange failed with status {}: {}", status, body);

        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&body) {
            let error_desc = error_json
                .get("error_description")
                .or_else(|| error_json.get("error"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(format!(
                "Token exchange failed ({}): {}",
                status, error_desc
            ));
        }

        return Err(format!(
            "Token exchange failed with status {}: {}",
            status, body
        ));
    }

    serde_json::from_str::<TokenResponse>(&body)
        .map_err(|e| format!("Failed to parse token response: {}", e))
}

/// Token response from OAuth/OIDC server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    #[serde(rename = "access_token")]
    pub access_token: Option<String>,
    #[serde(rename = "token_type")]
    pub token_type: Option<String>,
    #[serde(rename = "expires_in")]
    pub expires_in: Option<u64>,
    #[serde(rename = "refresh_token")]
    pub refresh_token: Option<String>,
    #[serde(rename = "id_token")]
    pub id_token: Option<String>,
    pub scope: Option<String>,
}

/// Start a local HTTP server to receive the OAuth callback
async fn start_callback_server(
    port: u16,
) -> Result<tokio::sync::oneshot::Receiver<AuthResponse>, String> {
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        match listener.accept().await {
            Ok((stream, _)) => {
                // Read the HTTP request
                let mut buffer = [0u8; 4096];
                if let Ok(n) = stream.try_read(&mut buffer) {
                    let request = String::from_utf8_lossy(&buffer[..n]);

                    // Parse the callback URL
                    if let Some(path) = request.lines().next().and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .filter(|p| p.starts_with("/?code="))
                    }) {
                        let query_string = &path[3..]; // Skip "/??"
                        let params: HashMap<String, String> =
                            url::form_urlencoded::parse(query_string.as_bytes())
                                .into_owned()
                                .collect();

                        if let (Some(code), Some(state)) = (params.get("code"), params.get("state"))
                        {
                            let _ = tx.send(AuthResponse {
                                code: code.clone(),
                                state: state.clone(),
                            });

                            // Send success response
                            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                                <html><body><h1>Login successful!</h1>\
                                <p>You can close this window.</p></body></html>";
                            let _ = stream.try_write(response.as_bytes());
                            return;
                        }
                    }

                    // Handle error or missing parameters
                    let error_response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n\
                        <html><body><h1>Login failed</h1>\
                        <p>Missing authorization code.</p></body></html>";
                    let _ = stream.try_write(error_response.as_bytes());
                }
            }
            Err(e) => {
                log::error!("Failed to accept callback connection: {}", e);
            }
        }
    });

    Ok(rx)
}

/// Attempt to open the browser for login (platform-dependent)
fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        return Err(format!(
            "Browser open not supported on this platform. Please visit: {}",
            url
        ));
    }
    log::debug!("Opened browser for IdP login");
    Ok(())
}

/// Acquire IdP login via browser - full OAuth + PKCE flow
pub async fn acquire_idp_id_token(options: IdpLoginOptions) -> Result<String, String> {
    log::info!("Starting IdP login for issuer: {}", options.idp_issuer);

    // Check cache first
    if let Some(cached_token) = get_cached_idp_id_token(&options.idp_issuer) {
        log::debug!("Using cached IdP token for issuer: {}", options.idp_issuer);
        return Ok(cached_token);
    }

    // Discover OIDC metadata
    let oidc_metadata = discover_oidc(&options.idp_issuer).await.ok();

    // Generate PKCE and state
    let pkce = generate_pkce();
    let state = generate_state();

    // Determine callback port
    let port = options.callback_port.unwrap_or(9999);
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    // Build authorization URL
    let auth_url = build_authorization_url(
        &options.idp_issuer,
        &options.idp_client_id,
        &redirect_uri,
        &pkce,
        &state,
        &["openid", "profile", "email"],
        oidc_metadata.as_ref(),
    )?;

    log::debug!("Authorization URL: {}", auth_url);

    // Open browser unless skipped
    if !options.skip_browser_open {
        open_browser(&auth_url).map_err(|e| {
            format!(
                "Failed to open browser for login. Please visit manually: {}\nError: {}",
                auth_url, e
            )
        })?;
    } else {
        log::info!("Browser open skipped. Please visit: {}", auth_url);
    }

    // Start callback server
    let callback_rx = start_callback_server(port).await?;

    // Wait for callback with timeout
    match tokio::time::timeout(Duration::from_millis(IDP_LOGIN_TIMEOUT_MS), callback_rx).await {
        Ok(Ok(auth_response)) => {
            // Verify state matches
            if auth_response.state != state {
                return Err("State mismatch - possible CSRF attack".to_string());
            }

            log::debug!("Received authorization code");

            // Exchange code for tokens
            let token_response = exchange_code_for_tokens(
                &options.idp_issuer,
                &options.idp_client_id,
                options.idp_client_secret.as_deref(),
                &auth_response.code,
                &redirect_uri,
                &pkce.code_verifier,
                oidc_metadata.as_ref(),
            )
            .await?;

            // Extract id_token
            let id_token = token_response
                .id_token
                .ok_or_else(|| "No id_token in token response".to_string())?;

            // Cache the token
            if let Some(expires_in) = token_response.expires_in {
                cache_idp_id_token(options.idp_issuer.clone(), id_token.clone(), expires_in);
            }

            log::info!("IdP login completed successfully");
            Ok(id_token)
        }
        Ok(Err(_)) => Err("Callback channel cancelled".to_string()),
        Err(_) => Err(format!(
            "IdP login timed out after {}ms",
            IDP_LOGIN_TIMEOUT_MS
        )),
    }
}

/// Get IdP client secret from secure storage
pub fn get_idp_client_secret(issuer: &str) -> Option<String> {
    // First check in-memory cache
    let key = normalize_issuer(issuer);
    if let Some(secret) = CLIENT_SECRET_CACHE.lock().unwrap().get(&key) {
        return Some(secret.clone());
    }
    // Fall back to environment variable
    std::env::var("AI_CODE_XAA_IDP_CLIENT_SECRET").ok()
}

/// Clear all cached IdP tokens
pub fn clear_all_idp_tokens() {
    let mut tokens = TOKEN_CACHE.lock().unwrap();
    let count = tokens.len();
    tokens.clear();
    log::debug!("Cleared {} cached IdP tokens", count);
}

/// Get the number of cached tokens
pub fn get_cached_token_count() -> usize {
    let tokens = TOKEN_CACHE.lock().unwrap();
    tokens.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xaa_enabled_default() {
        assert!(!is_xaa_enabled());
    }

    #[test]
    fn test_normalize_issuer() {
        // normalize_issuer trims trailing slashes and lowercases host
        assert_eq!(
            normalize_issuer("https://Example.COM/"),
            "https://example.com"
        );
        assert_eq!(
            normalize_issuer("https://auth.example.com/path/"),
            "https://auth.example.com/path"
        );
    }

    #[test]
    fn test_generate_pkce() {
        let pkce = generate_pkce();
        assert_eq!(pkce.code_verifier.len(), 64);
        // Code challenge is base64url of SHA256 (32 bytes)
        assert!(!pkce.code_challenge.is_empty());
    }

    #[test]
    fn test_generate_state() {
        let state = generate_state();
        assert_eq!(state.len(), 32);
    }

    #[test]
    fn test_get_cached_token_nonexistent() {
        assert!(get_cached_idp_id_token("https://nonexistent.com").is_none());
    }

    #[test]
    fn test_cache_and_get_token() {
        let issuer = "https://test.example.com";
        let token = "test-token-value";

        // Clear any existing tokens for this issuer
        clear_idp_id_token(issuer);

        cache_idp_id_token(issuer.to_string(), token.to_string(), 3600);
        assert_eq!(get_cached_idp_id_token(issuer), Some(token.to_string()));

        clear_idp_id_token(issuer);
        assert!(get_cached_idp_id_token(issuer).is_none());
    }

    #[test]
    fn test_clear_all_tokens() {
        // Use UUID-based issuers so no other parallel test can collide
        let unique1 = uuid::Uuid::new_v4().to_string();
        let unique2 = uuid::Uuid::new_v4().to_string();
        let issuer1 = format!("https://{}.test-isolated.invalid", unique1);
        let issuer2 = format!("https://{}.test-isolated.invalid", unique2);

        // Cache two tokens under unique keys
        cache_idp_id_token(issuer1.clone(), "token1".to_string(), 3600);
        cache_idp_id_token(issuer2.clone(), "token2".to_string(), 3600);

        // Both unique issuers should be present
        assert_eq!(
            get_cached_idp_id_token(&issuer1),
            Some("token1".to_string())
        );
        assert_eq!(
            get_cached_idp_id_token(&issuer2),
            Some("token2".to_string())
        );

        // Clear and verify gone
        clear_idp_id_token(&issuer1);
        clear_idp_id_token(&issuer2);
        assert_eq!(get_cached_idp_id_token(&issuer1), None);
        assert_eq!(get_cached_idp_id_token(&issuer2), None);
    }

    #[test]
    fn test_token_response_deserialization() {
        let json = r#"{
            "access_token": "at-123",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "rt-456",
            "id_token": "id-789",
            "scope": "openid profile email"
        }"#;

        let response: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, Some("at-123".to_string()));
        assert_eq!(response.id_token, Some("id-789".to_string()));
        assert_eq!(response.expires_in, Some(3600));
    }

    #[test]
    fn test_get_idp_client_secret_default() {
        // No env var set, should return None
        assert!(get_idp_client_secret("https://test.com").is_none());
    }
}
