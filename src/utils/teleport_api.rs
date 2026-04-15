#![allow(dead_code)]

pub const CCR_BYOC_BETA: &str = "ccr-byoc-2025-07-29";

pub fn is_transient_network_error(_error: &dyn std::error::Error) -> bool {
    false
}

pub async fn axios_get_with_retry<T>(
    _url: &str,
    _config: Option<reqwest::Client>,
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    RequiresAction,
    Running,
    Idle,
    Archived,
}

#[derive(Debug, Clone)]
pub struct GitSource {
    pub url: String,
    pub revision: Option<String>,
    pub allow_unrestricted_git_push: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub sources: Vec<GitSource>,
    pub cwd: String,
}

#[derive(Debug, Clone)]
pub struct SessionResource {
    pub id: String,
    pub title: Option<String>,
    pub session_status: SessionStatus,
    pub environment_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub session_context: SessionContext,
}

pub async fn prepare_api_request() -> Result<(String, String), Box<dyn std::error::Error>> {
    Err("Authentication required".into())
}

pub fn get_oauth_headers(access_token: &str) -> std::collections::HashMap<String, String> {
    let mut headers = std::collections::HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", access_token),
    );
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
    headers
}

pub fn get_branch_from_session(_session: &SessionResource) -> Option<String> {
    None
}
