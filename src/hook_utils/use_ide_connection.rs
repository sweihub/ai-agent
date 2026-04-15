use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct IdeConnectionState {
    connected: AtomicBool,
    last_check_ms: AtomicU64,
}

impl IdeConnectionState {
    pub fn new() -> Self {
        Self {
            connected: AtomicBool::new(false),
            last_check_ms: AtomicU64::new(now_timestamp()),
        }
    }

    pub fn set_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::SeqCst);
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    pub fn update_check_time(&self) {
        self.last_check_ms.store(now_timestamp(), Ordering::SeqCst);
    }

    pub fn time_since_last_check(&self) -> std::time::Duration {
        let last = self.last_check_ms.load(Ordering::SeqCst);
        let now = now_timestamp();
        std::time::Duration::from_millis(now.saturating_sub(last))
    }
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl Default for IdeConnectionState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error,
}

pub struct IdeConnectionManager {
    state: IdeConnectionState,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
}

impl IdeConnectionManager {
    pub fn new() -> Self {
        Self {
            state: IdeConnectionState::new(),
            reconnect_attempts: 0,
            max_reconnect_attempts: 3,
        }
    }

    pub fn with_max_reconnect(mut self, max: u32) -> Self {
        self.max_reconnect_attempts = max;
        self
    }

    pub fn connect(&mut self) -> ConnectionStatus {
        self.reconnect_attempts += 1;

        if self.reconnect_attempts <= self.max_reconnect_attempts {
            self.state.set_connected(true);
            self.reconnect_attempts = 0;
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Error
        }
    }

    pub fn disconnect(&self) {
        self.state.set_connected(false);
    }

    pub fn get_status(&self) -> ConnectionStatus {
        if self.state.is_connected() {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }

    pub fn can_reconnect(&self) -> bool {
        self.reconnect_attempts < self.max_reconnect_attempts
    }

    pub fn reset_attempts(&mut self) {
        self.reconnect_attempts = 0;
    }
}

impl Default for IdeConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_connection_state() {
        let state = IdeConnectionState::new();
        assert!(!state.is_connected());

        state.set_connected(true);
        assert!(state.is_connected());

        state.set_connected(false);
        assert!(!state.is_connected());
    }

    #[test]
    fn test_ide_connection_manager() {
        let mut manager = IdeConnectionManager::new();
        let status = manager.connect();
        assert_eq!(status, ConnectionStatus::Connected);

        manager.disconnect();
        assert_eq!(manager.get_status(), ConnectionStatus::Disconnected);
    }
}
