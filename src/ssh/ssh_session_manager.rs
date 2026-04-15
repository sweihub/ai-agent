use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct SSHSessionManager {
    sessions: Arc<RwLock<std::collections::HashMap<String, SSHSession>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SSHSession {
    pub session_id: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub created_at: i64,
    pub last_active: i64,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Connecting,
    Connected,
    Disconnected,
    Error,
}

impl SSHSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn create_session(&self, host: &str, user: &str, port: u16) -> SSHSession {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = SSHSession {
            session_id: session_id.clone(),
            host: host.to_string(),
            user: user.to_string(),
            port,
            created_at: chrono::Utc::now().timestamp_millis(),
            last_active: chrono::Utc::now().timestamp_millis(),
            status: SessionStatus::Connecting,
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, session.clone());
        session
    }

    pub async fn get_session(&self, session_id: &str) -> Option<SSHSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn remove_session(&self, session_id: &str) -> Option<SSHSession> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
    }

    pub async fn list_sessions(&self) -> Vec<SSHSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn update_status(&self, session_id: &str, status: SessionStatus) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = status;
            session.last_active = chrono::Utc::now().timestamp_millis();
        }
    }
}

impl Default for SSHSessionManager {
    fn default() -> Self {
        Self::new()
    }
}
