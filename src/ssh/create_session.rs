use std::collections::HashMap;

pub type SSHSession = HashMap<String, serde_json::Value>;

pub async fn create_ssh_session() -> SSHSession {
    HashMap::new()
}
