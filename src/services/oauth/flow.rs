// Source: /data/home/swei/claudecode/openclaudecode/src/services/oauth/index.ts
//! OAuth service that handles the OAuth 2.0 authorization code flow with PKCE.
//!
//! Supports two ways to get authorization codes:
//! 1. Automatic: Opens browser, redirects to localhost where we capture the code
//! 2. Manual: User manually copies and pastes the code (used in non-browser environments)
//!
//! Source: /data/home/swei/claudecode/openclaudecode/src/services/oauth/index.ts

use crate::services::oauth::auth_code_listener::AuthCodeListener;
use crate::services::oauth::client::{
    build_auth_url, exchange_code_for_tokens, parse_scopes, refresh_oauth_token,
};
use crate::services::oauth::crypto::{generate_code_challenge, generate_code_verifier, generate_state};
use crate::services::oauth::profile::get_oauth_profile_from_oauth_token;
use crate::services::oauth::types::{
    OAuthProfileResponse, OAuthTokenExchangeResponse, OAuthTokens, RateLimitTier,
    SubscriptionType, TokenAccount,
};

/// Callback type for handling auth URLs — the caller decides how to present them.
pub type AuthUrlHandler =
    Box<dyn FnOnce(String, Option<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>> + Send>;

/// Configuration for the OAuth flow.
#[derive(Debug, Default, Clone)]
pub struct OAuthFlowOptions {
    pub login_with_claude_ai: bool,
    pub inference_only: bool,
    pub expires_in: Option<u64>,
    pub org_uuid: Option<String>,
    pub login_hint: Option<String>,
    pub login_method: Option<String>,
    pub skip_browser_open: bool,
}

/// OAuth service that handles the OAuth 2.0 authorization code flow with PKCE.
pub struct OAuthService {
    code_verifier: String,
    auth_code_listener: Option<AuthCodeListener>,
    port: Option<u16>,
    options: OAuthFlowOptions,
}

impl OAuthService {
    pub fn new() -> Self {
        Self {
            code_verifier: generate_code_verifier(),
            auth_code_listener: None,
            port: None,
            options: OAuthFlowOptions::default(),
        }
    }

    /// Start the OAuth flow: create listener, generate PKCE values, build URLs,
    /// wait for authorization code, exchange for tokens, and fetch profile info.
    pub async fn start_oauth_flow(
        &mut self,
        auth_url_handler: AuthUrlHandler,
        options: OAuthFlowOptions,
    ) -> anyhow::Result<OAuthTokens> {
        self.options = options.clone();

        // Create OAuth callback listener and start it
        let listener = AuthCodeListener::new("/callback");
        let port = listener.start(None).await?;
        self.auth_code_listener = Some(listener);
        self.port = Some(port);

        // Generate PKCE values and state
        let code_challenge = generate_code_challenge(&self.code_verifier);
        let state = generate_state();

        // Build auth URLs for both automatic and manual flows
        let manual_flow_url = build_auth_url(
            &code_challenge,
            &state,
            port,
            true,
            options.login_with_claude_ai,
            options.inference_only,
            options.org_uuid.as_deref(),
            options.login_hint.as_deref(),
            options.login_method.as_deref(),
        );
        let automatic_flow_url = build_auth_url(
            &code_challenge,
            &state,
            port,
            false,
            options.login_with_claude_ai,
            options.inference_only,
            options.org_uuid.as_deref(),
            options.login_hint.as_deref(),
            options.login_method.as_deref(),
        );

        // Call on_ready to present URLs to the user
        let auth_url_handler = auth_url_handler;
        let manual_flow_url = manual_flow_url.clone();
        let automatic_flow_url = automatic_flow_url.clone();
        let skip_browser_open = options.skip_browser_open;

        auth_url_handler(manual_flow_url, Some(automatic_flow_url)).await?;

        // Wait for authorization code (from either automatic or manual)
        let listener = self.auth_code_listener.as_ref().ok_or_else(|| {
            anyhow::anyhow!("AuthCodeListener not initialized")
        })?;
        let state_str = state.clone();

        let authorization_code = listener.wait_for_authorization(state_str).await?;

        // Check if the automatic flow is still active (has a pending response)
        let is_automatic_flow = listener.has_pending_response().await;
        log::info!("OAuth auth code received: automatic={is_automatic_flow}");

        let port = self.port.unwrap_or(0);
        let use_manual_redirect = !is_automatic_flow;

        // Exchange authorization code for tokens
        let token_response = exchange_code_for_tokens(
            authorization_code.clone(),
            state.clone(),
            self.code_verifier.clone(),
            port,
            use_manual_redirect,
            options.expires_in,
        )
        .await?;

        // Fetch profile info
        let profile_info = get_oauth_profile_from_oauth_token(&token_response.access_token)
            .await
            .unwrap_or_default();

        // Handle success redirect for automatic flow
        if is_automatic_flow {
            let scopes = parse_scopes(token_response.scope.as_deref());
            if let Some(ref listener) = self.auth_code_listener {
                listener.handle_success_redirect(&scopes).await;
            }
        }

        let result = self.format_tokens(&token_response, &profile_info);

        // Always cleanup
        if let Some(ref listener) = self.auth_code_listener {
            listener.close().await;
        }

        result
    }

    /// Handle manual flow callback when user pastes the auth code.
    /// Call this when the user provides an auth code manually (non-browser flow).
    pub async fn handle_manual_auth_code_input(&self, authorization_code: String) -> anyhow::Result<()> {
        if let Some(ref listener) = self.auth_code_listener {
            listener.send_manual_code(authorization_code).await?;
            // Close the auth code listener since manual input was used
            listener.close().await;
        }
        Ok(())
    }

    /// Exchange an authorization code for OAuth tokens.
    pub async fn exchange_code(
        &self,
        authorization_code: String,
        state: String,
    ) -> anyhow::Result<OAuthTokens> {
        let port = self.port.ok_or_else(|| anyhow::anyhow!("Port not set"))?;

        let token_response = exchange_code_for_tokens(
            authorization_code,
            state,
            self.code_verifier.clone(),
            port,
            true,
            None,
        )
        .await?;

        let profile_info = get_oauth_profile_from_oauth_token(&token_response.access_token)
            .await
            .unwrap_or_default();

        self.format_tokens(&token_response, &profile_info)
    }

    /// Refresh OAuth tokens.
    pub async fn refresh_tokens(&self, refresh_token: String) -> anyhow::Result<OAuthTokens> {
        refresh_oauth_token(refresh_token, None).await
    }

    /// Format the token exchange response into OAuthTokens.
    fn format_tokens(
        &self,
        response: &OAuthTokenExchangeResponse,
        profile: &OAuthProfileResponse,
    ) -> anyhow::Result<OAuthTokens> {
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64 + (response.expires_in as i64 * 1000))
            .unwrap_or(0);

        let token_account = response.account.as_ref().map(|a| TokenAccount {
            uuid: a.uuid.clone(),
            email_address: a.email_address.clone(),
            organization_uuid: a.organization_uuid.clone(),
        });

        let subscription_type = profile
            .extra
            .get("organization")
            .and_then(|v| v.get("organization_type"))
            .and_then(|v| v.as_str())
            .and_then(|org_type| match org_type {
                "claude_max" => Some("max".to_string()),
                "claude_pro" => Some("pro".to_string()),
                "claude_enterprise" => Some("enterprise".to_string()),
                "claude_team" => Some("team".to_string()),
                _ => None,
            });

        let rate_limit_tier = profile
            .extra
            .get("organization")
            .and_then(|v| v.get("rate_limit_tier"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(OAuthTokens {
            access_token: Some(response.access_token.clone()),
            refresh_token: response.refresh_token.clone(),
            expires_at: Some(expires_at),
            expires_in: Some(response.expires_in),
            scopes: parse_scopes(response.scope.as_deref()),
            subscription_type,
            rate_limit_tier,
            profile: Some(profile.clone()),
            token_account,
            ..OAuthTokens::default()
        })
    }

    /// Clean up any resources (like the local server).
    pub async fn cleanup(&mut self) {
        if let Some(ref listener) = self.auth_code_listener {
            listener.close().await;
        }
        self.auth_code_listener = None;
        self.port = None;
    }
}

impl Drop for OAuthService {
    fn drop(&mut self) {
        log::debug!("OAuthService dropped (use async cleanup() for graceful shutdown)");
    }
}
