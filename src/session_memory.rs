//! Session memory - backward-compatible re-export of [services::session_memory].
//!
//! The actual implementation lives in `src/services/session_memory/`.
//! This module provides:
//! - Re-exports of all new module symbols
//! - Backward-compatible adapters for callers that expect legacy types

// Re-export everything from the new module
pub use crate::services::session_memory::*;

/// Count tool calls in assistant messages since a given index.
/// Backward-compatible wrapper around the new module's internal function.
pub fn count_tool_calls_since(
    messages: &[crate::types::Message],
    since_index: Option<usize>,
) -> usize {
    crate::services::session_memory::session_memory::count_tool_calls_since(
        messages,
        since_index,
    )
}

/// Get the last summarized message as a message *index* into the provided
/// message slice. Returns the length of the slice minus one when we only
/// have a string UUID (the index is not recoverable).
///
/// This adapter exists because the old API returned `Option<usize>` (index)
/// while the new API stores `Option<String>` (UUID).
pub fn get_last_summarized_message_id_as_index(messages: &[crate::types::Message]) -> Option<usize> {
    let uuid = get_last_summarized_message_id();
    match uuid {
        Some(ref id) => {
            // Find the message index matching this UUID.
            messages.iter().position(|m| m.uuid.as_deref() == Some(id)).or_else(|| {
                if messages.is_empty() {
                    None
                } else {
                    Some(messages.len() - 1)
                }
            })
        }
        None => None,
    }
}
