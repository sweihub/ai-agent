use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::WebSocketStream;

#[derive(Debug, Clone)]
pub struct SessionsWebSocket {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    message_sender: mpsc::UnboundedSender<WsMessage>,
}

#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub session_id: String,
    pub url: String,
    pub status: ConnectionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsMessage {
    pub session_id: String,
    pub message_type: WsMessageType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsMessageType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

impl SessionsWebSocket {
    pub fn new() -> Self {
        let (message_sender, _) = mpsc::unbounded_channel();
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            message_sender,
        }
    }

    pub async fn add_connection(&self, session_id: &str, url: &str) {
        let mut connections = self.connections.write().await;
        connections.insert(
            session_id.to_string(),
            WebSocketConnection {
                session_id: session_id.to_string(),
                url: url.to_string(),
                status: ConnectionStatus::Connecting,
            },
        );
    }

    pub async fn remove_connection(&self, session_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(session_id);
    }

    pub async fn get_connection(&self, session_id: &str) -> Option<WebSocketConnection> {
        let connections = self.connections.read().await;
        connections.get(session_id).cloned()
    }

    pub async fn list_connections(&self) -> Vec<WebSocketConnection> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    pub async fn update_status(&self, session_id: &str, status: ConnectionStatus) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(session_id) {
            conn.status = status;
        }
    }
}

impl Default for SessionsWebSocket {
    fn default() -> Self {
        Self::new()
    }
}
