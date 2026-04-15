#![allow(dead_code)]

pub async fn start_remote_session(
    _session_id: &str,
) -> Result<RemoteSession, Box<dyn std::error::Error>> {
    Err("Not implemented".into())
}

pub struct RemoteSession {
    pub session_id: String,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum SessionStatus {
    Connecting,
    Connected,
    Disconnected,
}
