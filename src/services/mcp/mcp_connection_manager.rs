// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/useManageMCPConnections.ts
//! MCP connection manager
//! Manages MCP server connections with automatic reconnection

/// Constants for reconnection with exponential backoff
pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;
pub const INITIAL_BACKOFF_MS: u64 = 1000;
pub const MAX_BACKOFF_MS: u64 = 30000;

/// MCP batch flush timing (milliseconds)
pub const MCP_BATCH_FLUSH_MS: u64 = 16;

/// Calculate backoff delay for reconnection attempt
pub fn calculate_backoff_ms(attempt: u32) -> u64 {
    let delay = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
    delay.min(MAX_BACKOFF_MS)
}

/// Get transport display name for logging
pub fn get_transport_display_name(config_type: &str) -> &'static str {
    match config_type {
        "http" => "HTTP",
        "ws" | "ws-ide" => "WebSocket",
        _ => "SSE",
    }
}

/// Reconnection state for an MCP server
#[derive(Debug, Clone)]
pub struct ReconnectionState {
    pub attempt: u32,
    pub max_attempts: u32,
    pub start_time_ms: u64,
}

impl ReconnectionState {
    pub fn new() -> Self {
        Self {
            attempt: 0,
            max_attempts: MAX_RECONNECT_ATTEMPTS,
            start_time_ms: 0,
        }
    }

    pub fn can_retry(&self) -> bool {
        self.attempt < self.max_attempts
    }

    pub fn next_attempt(&mut self) {
        self.attempt += 1;
    }
}

impl Default for ReconnectionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_backoff() {
        assert_eq!(calculate_backoff_ms(1), 1000);
        assert_eq!(calculate_backoff_ms(2), 2000);
        assert_eq!(calculate_backoff_ms(3), 4000);
        // Should cap at MAX_BACKOFF_MS
        assert_eq!(calculate_backoff_ms(10), MAX_BACKOFF_MS);
    }

    #[test]
    fn test_get_transport_display_name() {
        assert_eq!(get_transport_display_name("http"), "HTTP");
        assert_eq!(get_transport_display_name("ws"), "WebSocket");
        assert_eq!(get_transport_display_name("ws-ide"), "WebSocket");
        assert_eq!(get_transport_display_name("stdio"), "SSE");
    }

    #[test]
    fn test_reconnection_state() {
        let mut state = ReconnectionState::new();
        assert!(state.can_retry());

        state.next_attempt();
        assert_eq!(state.attempt, 1);

        // Exhaust attempts
        for _ in 1..MAX_RECONNECT_ATTEMPTS {
            state.next_attempt();
        }
        assert!(!state.can_retry());
    }
}
