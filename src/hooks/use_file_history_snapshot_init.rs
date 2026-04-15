// Source: ~/claudecode/openclaudecode/src/hooks/useFileHistorySnapshotInit.ts
#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};

/// A snapshot of file history at a point in time.
#[derive(Debug, Clone)]
pub struct FileHistorySnapshot {
    pub file_path: String,
    pub content: String,
    pub timestamp: u64,
}

/// The current state of file history.
#[derive(Debug, Clone, Default)]
pub struct FileHistoryState {
    pub snapshots: Vec<FileHistorySnapshot>,
    pub enabled: bool,
}

/// Check if file history is enabled.
pub fn file_history_enabled() -> bool {
    // Translation of `fileHistoryEnabled()` from the TypeScript module.
    // Uses AI_CODE_ prefix instead of CLAUDE_CODE_.
    crate::env_utils::is_env_truthy(&std::env::var("AI_CODE_FILE_HISTORY").unwrap_or_default())
}

/// Restore file history state from a log of snapshots.
pub fn file_history_restore_state_from_log(
    snapshots: &[FileHistorySnapshot],
    on_update_state: &dyn Fn(FileHistoryState),
) {
    let state = FileHistoryState {
        snapshots: snapshots.to_vec(),
        enabled: true,
    };
    on_update_state(state);
}

/// Initialize file history snapshot state on first call.
///
/// Translation of the React `useFileHistorySnapshotInit` hook.
/// In Rust this is a plain initialization function with a guard
/// to ensure it only runs once (equivalent to the `useRef` initialized flag).
pub fn file_history_snapshot_init(
    initial_file_history_snapshots: Option<&[FileHistorySnapshot]>,
    on_update_state: &dyn Fn(FileHistoryState),
) {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if !file_history_enabled() || INITIALIZED.load(Ordering::SeqCst) {
        return;
    }
    INITIALIZED.store(true, Ordering::SeqCst);

    if let Some(snapshots) = initial_file_history_snapshots {
        file_history_restore_state_from_log(snapshots, on_update_state);
    }
}
