// Source: /data/home/swei/claudecode/openclaudecode/src/utils/sessionState.ts
//! Session state machine for tracking agent lifecycle.
//!
//! Mirrors the TypeScript sessionState.ts with three states:
//! - Idle: agent not running
//! - Running: agent loop active
//! - RequiresAction: waiting for user input (permission, question)

use std::sync::atomic::{AtomicU32, Ordering};

/// Session state machine states
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SessionState {
    #[default]
    Idle,
    Running,
    RequiresAction { details: Option<RequiresActionDetails> },
}

impl SessionState {
    pub fn as_str(&self) -> &str {
        match self {
            SessionState::Idle => "idle",
            SessionState::Running => "running",
            SessionState::RequiresAction { .. } => "requires_action",
        }
    }
}

/// Details about why the session requires user action
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiresActionDetails {
    pub typ: ActionType,
    pub permission_denial: Option<PermissionDenialInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    Permission,
    Question,
    Interrupt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionDenialInfo {
    pub tool_name: String,
    pub tool_use_id: String,
}

/// Thread-safe session state tracker
#[derive(Debug, Default)]
pub struct SessionStateManager {
    state: std::sync::Mutex<SessionState>,
    permission_denial_count: AtomicU32,
}

impl SessionStateManager {
    pub fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(SessionState::Idle),
            permission_denial_count: AtomicU32::new(0),
        }
    }

    pub fn get_state(&self) -> SessionState {
        self.state.lock().unwrap().clone()
    }

    pub fn set_state(&self, state: SessionState) {
        *self.state.lock().unwrap() = state;
    }

    pub fn start_running(&self) {
        *self.state.lock().unwrap() = SessionState::Running;
    }

    pub fn stop(&self) {
        *self.state.lock().unwrap() = SessionState::Idle;
    }

    pub fn require_action(&self, details: RequiresActionDetails) {
        *self.state.lock().unwrap() =
            SessionState::RequiresAction {
                details: Some(details),
            };
    }

    pub fn clear_action(&self) {
        *self.state.lock().unwrap() = SessionState::Idle;
    }

    pub fn permission_denial_count(&self) -> u32 {
        self.permission_denial_count.load(Ordering::Relaxed)
    }

    pub fn increment_permission_denial(&self) {
        self.permission_denial_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn reset_permission_denial(&self) {
        self.permission_denial_count.store(0, Ordering::Relaxed);
    }

    /// Check if permission denial count indicates tool is consistently denied
    pub fn is_consistently_denied(&self, threshold: u32) -> bool {
        self.permission_denial_count.load(Ordering::Relaxed) >= threshold
    }
}

impl Clone for SessionStateManager {
    fn clone(&self) -> Self {
        let state = self.state.lock().unwrap().clone();
        Self {
            state: std::sync::Mutex::new(state),
            permission_denial_count: AtomicU32::new(self.permission_denial_count.load(Ordering::Relaxed)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_transitions() {
        let manager = SessionStateManager::new();

        // Initial state is idle
        assert_eq!(manager.get_state(), SessionState::Idle);

        // Transition to running
        manager.start_running();
        assert_eq!(manager.get_state(), SessionState::Running);

        // Transition to requires_action
        manager.require_action(RequiresActionDetails {
            typ: ActionType::Permission,
            permission_denial: Some(PermissionDenialInfo {
                tool_name: "Bash".to_string(),
                tool_use_id: "test-123".to_string(),
            }),
        });
        assert_eq!(manager.get_state().as_str(), "requires_action");

        // Clear action -> idle
        manager.clear_action();
        assert_eq!(manager.get_state(), SessionState::Idle);

        // Stop -> idle
        manager.stop();
        assert_eq!(manager.get_state(), SessionState::Idle);
    }

    #[test]
    fn test_permission_denial_count() {
        let manager = SessionStateManager::new();
        assert_eq!(manager.permission_denial_count(), 0);
        assert!(!manager.is_consistently_denied(3));

        manager.increment_permission_denial();
        manager.increment_permission_denial();
        assert_eq!(manager.permission_denial_count(), 2);
        assert!(!manager.is_consistently_denied(3));

        manager.increment_permission_denial();
        assert!(manager.is_consistently_denied(3));

        manager.reset_permission_denial();
        assert_eq!(manager.permission_denial_count(), 0);
    }

    #[test]
    fn test_session_state_as_str() {
        assert_eq!(SessionState::Idle.as_str(), "idle");
        assert_eq!(SessionState::Running.as_str(), "running");
        assert_eq!(
            SessionState::RequiresAction { details: None }.as_str(),
            "requires_action"
        );
    }
}
