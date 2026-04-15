use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTokenResult {
    pub github_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportTokenError {
    pub kind: String,
    pub status: Option<u16>,
}

pub struct RedactedGithubToken(String);

impl RedactedGithubToken {
    pub fn new(raw: String) -> Self {
        Self(raw)
    }

    pub fn reveal(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RedactedGithubToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED:gh-token]")
    }
}

pub async fn import_github_token(
    _token: RedactedGithubToken,
) -> Result<ImportTokenResult, ImportTokenError> {
    Err(ImportTokenError {
        kind: "not_signed_in".to_string(),
        status: None,
    })
}

pub async fn create_default_environment() -> bool {
    false
}

pub async fn is_signed_in() -> bool {
    false
}

pub fn get_code_web_url() -> String {
    "https://claude.ai/code".to_string()
}
