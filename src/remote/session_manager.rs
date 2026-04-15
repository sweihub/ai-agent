use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RemoteSessionManager {
    sessions: Arc<RwLock<std::collections::HashMap<String, RemoteSession>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSession {
    pub session_id: String,
    pub project_path: String,
    pub created_at: i64,
    pub last_active: i64,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Idle,
    Disconnected,
    Terminated,
}

impl RemoteSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn add_session(&self, session: RemoteSession) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session);
    }

    pub async fn remove_session(&self, session_id: &str) -> Option<RemoteSession> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
    }

    pub async fn get_session(&self, session_id: &str) -> Option<RemoteSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn list_sessions(&self) -> Vec<RemoteSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn update_last_active(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_active = chrono::Utc::now().timestamp_millis();
        }
    }
}

impl Default for RemoteSessionManager {
    fn default() -> Self {
        Self::new()
    }
}
