use super::types::{SessionInfo, SessionState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct DirectConnectManager {
    sessions: Arc<RwLock<std::collections::HashMap<String, DirectConnectSession>>>,
    config: Arc<RwLock<super::types::ServerConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectConnectSession {
    pub session_id: String,
    pub project_path: String,
    pub created_at: i64,
    pub last_active: i64,
    pub status: SessionState,
    pub ws_url: Option<String>,
    pub auth_token: Option<String>,
}

impl DirectConnectManager {
    pub fn new(config: super::types::ServerConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn create_session(&self, project_path: &str) -> Result<DirectConnectSession, String> {
        let mut sessions = self.sessions.write().await;
        let session_id = uuid::Uuid::new_v4().to_string();

        let session = DirectConnectSession {
            session_id: session_id.clone(),
            project_path: project_path.to_string(),
            created_at: chrono::Utc::now().timestamp_millis(),
            last_active: chrono::Utc::now().timestamp_millis(),
            status: SessionState::Starting,
            ws_url: None,
            auth_token: None,
        };

        sessions.insert(session_id, session.clone());
        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<DirectConnectSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn list_sessions(&self) -> Vec<DirectConnectSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn remove_session(&self, session_id: &str) -> Option<DirectConnectSession> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
    }

    pub async fn update_status(&self, session_id: &str, status: SessionState) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = status;
            session.last_active = chrono::Utc::now().timestamp_millis();
        }
    }

    pub async fn get_config(&self) -> super::types::ServerConfig {
        let config = self.config.read().await;
        config.clone()
    }
}
