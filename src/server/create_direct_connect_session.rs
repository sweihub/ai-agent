use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDirectConnectSessionRequest {
    pub project_path: String,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub auto_mode: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDirectConnectSessionResponse {
    pub session_id: String,
    pub ws_url: String,
    pub auth_token: String,
    #[serde(default)]
    pub work_dir: Option<String>,
}

pub fn generate_session_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("dc_{:x}", timestamp)
}

pub fn validate_project_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Project path cannot be empty".to_string());
    }

    let path_obj = std::path::Path::new(path);
    if !path_obj.exists() {
        return Err(format!("Project path does not exist: {}", path));
    }

    if !path_obj.is_dir() {
        return Err(format!("Project path is not a directory: {}", path));
    }

    Ok(())
}
