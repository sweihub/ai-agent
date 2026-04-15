use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupContext {
    #[serde(rename = "useCurrentRepo")]
    pub use_current_repo: Option<bool>,
    #[serde(rename = "workflowExists")]
    pub workflow_exists: Option<bool>,
    #[serde(rename = "secretExists")]
    pub secret_exists: Option<bool>,
}

pub async fn setup_github_actions(
    _repo_name: &str,
    _api_key_or_oauth_token: Option<&str>,
    _secret_name: &str,
    _update_progress: impl Fn(),
    _skip_workflow: bool,
    _selected_workflows: Vec<&str>,
    _auth_type: &str,
    _context: Option<SetupContext>,
) -> Result<(), String> {
    Ok(())
}
