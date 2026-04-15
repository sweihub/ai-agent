use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSSHSessionRequest {
    pub host: String,
    pub user: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub key_path: Option<String>,
    pub password: Option<String>,
}

fn default_port() -> u16 {
    22
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSSHSessionResponse {
    pub session_id: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub status: String,
}

pub fn validate_ssh_host(host: &str) -> Result<(), String> {
    if host.is_empty() {
        return Err("SSH host cannot be empty".to_string());
    }
    Ok(())
}

pub fn validate_ssh_user(user: &str) -> Result<(), String> {
    if user.is_empty() {
        return Err("SSH user cannot be empty".to_string());
    }
    Ok(())
}
