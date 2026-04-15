// Source: /data/home/swei/claudecode/openclaudecode/src/utils/sessionRestore.ts
//! Session restore utilities - restore sessions from file-based storage

use crate::utils::session_storage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result of session restore operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRestoreResult {
    pub session_id: String,
    pub message_count: usize,
    pub success: bool,
    pub error: Option<String>,
}

/// Extended restore result with transcript data
#[derive(Debug, Clone)]
pub struct SessionRestoreData {
    pub result: SessionRestoreResult,
    pub transcript: Vec<String>,
}

/// Attempt to restore a session from storage
pub fn restore_session(session_id: &str) -> SessionRestoreResult {
    // Check if session exists
    if !session_storage::session_exists(session_id) {
        return SessionRestoreResult {
            session_id: session_id.to_string(),
            message_count: 0,
            success: false,
            error: Some("Session not found".to_string()),
        };
    }

    // Load transcript
    let messages = session_storage::load_transcript(session_id);
    let message_count = messages.len();

    if message_count == 0 {
        return SessionRestoreResult {
            session_id: session_id.to_string(),
            message_count: 0,
            success: false,
            error: Some("Session transcript is empty".to_string()),
        };
    }

    SessionRestoreResult {
        session_id: session_id.to_string(),
        message_count,
        success: true,
        error: None,
    }
}

/// Restore a session and return the transcript data
pub fn restore_session_with_data(session_id: &str) -> SessionRestoreData {
    let result = restore_session(session_id);
    let transcript = if result.success {
        session_storage::load_transcript(session_id)
    } else {
        vec![]
    };

    SessionRestoreData { result, transcript }
}

/// Check if a session can be restored
pub fn can_restore_session(session_id: &str) -> bool {
    // Check if session data exists and is valid
    session_storage::session_exists(session_id)
        && session_storage::is_session_data_valid(session_id)
}

/// Get the path to a session's transcript file
pub fn get_session_transcript_path(session_id: &str) -> PathBuf {
    session_storage::get_transcript_path(session_id)
}

/// List all restorable sessions
pub fn list_restorable_sessions() -> Vec<String> {
    session_storage::list_stored_sessions()
        .into_iter()
        .filter(|id| can_restore_session(id))
        .collect()
}

/// Get the number of messages in a stored session without fully loading it
pub fn get_stored_message_count(session_id: &str) -> Option<usize> {
    if !session_storage::session_exists(session_id) {
        return None;
    }

    let transcript = session_storage::load_transcript(session_id);
    if transcript.is_empty() {
        None
    } else {
        Some(transcript.len())
    }
}

/// Attempt to restore the most recent session
pub fn restore_latest_session() -> Option<SessionRestoreResult> {
    let sessions = list_restorable_sessions();
    if sessions.is_empty() {
        return None;
    }

    // Return the first (most recently modified) session
    let latest = sessions.first()?;
    Some(restore_session(latest))
}

/// Delete a restored session
pub fn delete_restored_session(session_id: &str) -> Result<(), String> {
    if !can_restore_session(session_id) {
        return Err(format!("Cannot delete session: {}", session_id));
    }

    session_storage::delete_session_storage(session_id)
}

/// Get the storage size of a session in bytes
pub fn get_session_storage_size(session_id: &str) -> u64 {
    session_storage::get_transcript_size(session_id)
}

/// Validate that a session's transcript is not corrupted
pub fn validate_session_integrity(session_id: &str) -> Result<(), String> {
    if !session_storage::session_exists(session_id) {
        return Err("Session does not exist".to_string());
    }

    if !session_storage::is_session_data_valid(session_id) {
        return Err("Session data is corrupted".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_nonexistent_session() {
        let result = restore_session("nonexistent-session");
        assert!(!result.success);
        assert_eq!(result.message_count, 0);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_can_restore_nonexistent_session() {
        assert!(!can_restore_session("nonexistent-session"));
    }

    #[test]
    fn test_list_restorable_sessions_empty() {
        let sessions = list_restorable_sessions();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_get_stored_message_count_nonexistent() {
        assert!(get_stored_message_count("nonexistent").is_none());
    }

    #[test]
    fn test_restore_latest_no_sessions() {
        assert!(restore_latest_session().is_none());
    }

    #[test]
    fn test_delete_nonexistent_session() {
        let result = delete_restored_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_nonexistent_session() {
        let result = validate_session_integrity("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_session_storage_size_nonexistent() {
        assert_eq!(get_session_storage_size("nonexistent"), 0);
    }
}
