//! Portable session storage utilities for moving sessions between machines.

use std::path::PathBuf;

/// Export session to a portable format
pub fn export_session(session_id: &str, target_dir: &PathBuf) -> Result<PathBuf, String> {
    // Create export directory
    let export_dir = target_dir.join(format!("session_{}", session_id));
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    // Copy transcript
    let transcript_src = crate::utils::session_storage::get_transcript_path(session_id);
    let transcript_dst = export_dir.join("transcript.jsonl");

    if transcript_src.exists() {
        std::fs::copy(&transcript_src, &transcript_dst).map_err(|e| e.to_string())?;
    }

    // Copy state
    let state_src = crate::utils::session_storage::get_session_state_path(session_id);
    let state_dst = export_dir.join("state.json");

    if state_src.exists() {
        std::fs::copy(&state_src, &state_dst).map_err(|e| e.to_string())?;
    }

    Ok(export_dir)
}

/// Import session from a portable format
pub fn import_session(export_dir: &PathBuf) -> Result<String, String> {
    // Read session ID from export directory name
    let session_id = export_dir
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix("session_"))
        .ok_or("Invalid export directory name")?
        .to_string();

    // Copy transcript
    let transcript_src = export_dir.join("transcript.jsonl");
    let transcript_dst = crate::utils::session_storage::get_transcript_path(&session_id);

    if transcript_src.exists() {
        // Ensure parent directory exists
        if let Some(parent) = transcript_dst.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&transcript_src, &transcript_dst).map_err(|e| e.to_string())?;
    }

    // Copy state
    let state_src = export_dir.join("state.json");
    let state_dst = crate::utils::session_storage::get_session_state_path(&session_id);

    if state_src.exists() {
        std::fs::copy(&state_src, &state_dst).map_err(|e| e.to_string())?;
    }

    Ok(session_id)
}
