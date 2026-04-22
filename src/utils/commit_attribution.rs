//! Commit attribution utilities for tracking Claude's contributions.
//!
//! This module provides functionality to track and attribute file changes
//! to Claude or human contributors for git commit attribution.

use crate::constants::env::ai;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Types
// ============================================================================

/// Attribution state for tracking Claude's contributions to files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionState {
    /// File states keyed by relative path (from cwd)
    pub file_states: HashMap<String, FileAttributionState>,
    /// Session baseline states for net change calculation
    pub session_baselines: HashMap<String, SessionBaseline>,
    /// Surface from which edits were made
    pub surface: String,
    /// HEAD SHA at session start (for detecting external commits)
    pub starting_head_sha: Option<String>,
    /// Total prompts in session (for steer count calculation)
    pub prompt_count: u32,
    /// Prompts at last commit (to calculate steers for current commit)
    pub prompt_count_at_last_commit: u32,
    /// Permission prompt tracking
    pub permission_prompt_count: u32,
    pub permission_prompt_count_at_last_commit: u32,
    /// ESC press tracking (user cancelled permission prompt)
    pub escape_count: u32,
    pub escape_count_at_last_commit: u32,
}

/// Per-file attribution state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileAttributionState {
    pub content_hash: String,
    pub claude_contribution: u64,
    pub mtime: u64,
}

/// Session baseline for tracking file state at session start.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBaseline {
    pub content_hash: String,
    pub mtime: u64,
}

/// Summary of Claude's contribution for a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionSummary {
    pub claude_percent: u32,
    pub claude_chars: u64,
    pub human_chars: u64,
    pub surfaces: Vec<String>,
}

/// Per-file attribution details for git notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttribution {
    pub claude_chars: u64,
    pub human_chars: u64,
    pub percent: u32,
    pub surface: String,
}

/// Full attribution data for git notes JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionData {
    pub version: u32,
    pub summary: AttributionSummary,
    pub files: HashMap<String, FileAttribution>,
    pub surface_breakdown: HashMap<String, SurfaceBreakdown>,
    pub excluded_generated: Vec<String>,
    pub sessions: Vec<String>,
}

/// Surface breakdown for attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceBreakdown {
    pub claude_chars: u64,
    pub percent: u32,
}

/// Attribution snapshot message for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionSnapshotMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub message_id: String,
    pub surface: String,
    pub file_states: HashMap<String, FileAttributionState>,
    pub prompt_count: u32,
    pub prompt_count_at_last_commit: u32,
    pub permission_prompt_count: u32,
    pub permission_prompt_count_at_last_commit: u32,
    pub escape_count: u32,
    pub escape_count_at_last_commit: u32,
}

// ============================================================================
// Constants
// ============================================================================

/// List of repos where internal model names are allowed in trailers.
/// Includes both SSH and HTTPS URL formats.
const INTERNAL_MODEL_REPOS: &[&str] = &[
    "github.com:anthropics/claude-cli-internal",
    "github.com/anthropics/claude-cli-internal",
    "github.com:anthropics/anthropic",
    "github.com/anthropics/anthropic",
    "github.com:anthropics/apps",
    "github.com/anthropics/apps",
    "github.com:anthropics/casino",
    "github.com/anthropics/casino",
    "github.com:anthropics/dbt",
    "github.com/anthropics/dbt",
    "github.com:anthropics/dotfiles",
    "github.com/anthropics/dotfiles",
    "github.com:anthropics/terraform-config",
    "github.com/anthropics/terraform-config",
    "github.com:anthropics/hex-export",
    "github.com/anthropics/hex-export",
    "github.com:anthropics/feedback-v2",
    "github.com/anthropics/feedback-v2",
    "github.com:anthropics/labs",
    "github.com/anthropics/labs",
    "github.com:anthropics/argo-rollouts",
    "github.com/anthropics/argo-rollouts",
    "github.com:anthropics/starling-configs",
    "github.com/anthropics/starling-configs",
    "github.com:anthropics/ts-tools",
    "github.com/anthropics/ts-tools",
    "github.com:anthropics/ts-capsules",
    "github.com/anthropics/ts-capsules",
    "github.com:anthropics/feldspar-testing",
    "github.com/anthropics/feldspar-testing",
    "github.com:anthropics/trellis",
    "github.com/anthropics/trellis",
    "github.com:anthropics/claude-for-hiring",
    "github.com/anthropics/claude-for-hiring",
    "github.com:anthropics/forge-web",
    "github.com/anthropics/forge-web",
    "github.com:anthropics/infra-manifests",
    "github.com/anthropics/infra-manifests",
    "github.com:anthropics/mycro_manifests",
    "github.com/anthropics/mycro_manifests",
    "github.com:anthropics/mycro_configs",
    "github.com/anthropics/mycro_configs",
    "github.com:anthropics/mobile-apps",
    "github.com/anthropics/mobile-apps",
];

// ============================================================================
// Cache for repo classification
// ============================================================================

/// Cache for repo classification result. Primed once per process.
/// 'internal' = remote matches INTERNAL_MODEL_REPOS allowlist
/// 'external' = has a remote, not on allowlist (public/open-source repo)
/// 'none'     = no remote URL (not a git repo, or no remote configured)
lazy_static::lazy_static! {
    static ref REPO_CLASS_CACHE: RwLock<Option<RepoClass>> = RwLock::new(None);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepoClass {
    Internal,
    External,
    None,
}

// ============================================================================
// Public Functions
// ============================================================================

/// Get the repo root for attribution operations.
/// Uses get_cwd() which respects agent worktree overrides,
/// then resolves to git root to handle `cd subdir` case.
/// Falls back to get_original_cwd() if git root can't be determined.
pub fn get_attribution_repo_root() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            crate::utils::get_original_cwd()
                .to_string_lossy()
                .to_string()
        })
}

/// Synchronously return the cached repo classification.
/// Returns None if the async check hasn't run yet.
pub fn get_repo_class_cached() -> Option<String> {
    REPO_CLASS_CACHE.read().ok().and_then(|guard| {
        guard.map(|c| match c {
            RepoClass::Internal => "internal".to_string(),
            RepoClass::External => "external".to_string(),
            RepoClass::None => "none".to_string(),
        })
    })
}

/// Synchronously return the cached result of is_internal_model_repo().
/// Returns false if the check hasn't run yet (safe default: don't leak).
pub fn is_internal_model_repo_cached() -> bool {
    REPO_CLASS_CACHE
        .read()
        .ok()
        .map(|guard| *guard == Some(RepoClass::Internal))
        .unwrap_or(false)
}

/// Sanitize a surface key to use public model names.
/// Converts internal model variants to their public equivalents.
pub fn sanitize_surface_key(surface_key: &str) -> String {
    // Split surface key into surface and model parts (e.g., "cli/opus-4-5-fast" -> ["cli", "opus-4-5-fast"])
    if let Some(slash_index) = surface_key.rfind('/') {
        let surface = &surface_key[..slash_index];
        let model = &surface_key[slash_index + 1..];
        let sanitized_model = sanitize_model_name(model);
        format!("{}/{}", surface, sanitized_model)
    } else {
        surface_key.to_string()
    }
}

/// Sanitize a model name to its public equivalent.
/// Maps internal variants to their public names based on model family.
pub fn sanitize_model_name(short_name: &str) -> String {
    // Map internal variants to public equivalents based on model family
    if short_name.contains("opus-4-6") {
        return "claude-opus-4-6".to_string();
    }
    if short_name.contains("opus-4-5") {
        return "claude-opus-4-5".to_string();
    }
    if short_name.contains("opus-4-1") {
        return "claude-opus-4-1".to_string();
    }
    if short_name.contains("opus-4") {
        return "claude-opus-4".to_string();
    }
    if short_name.contains("sonnet-4-6") {
        return "claude-sonnet-4-6".to_string();
    }
    if short_name.contains("sonnet-4-5") {
        return "claude-sonnet-4-5".to_string();
    }
    if short_name.contains("sonnet-4") {
        return "claude-sonnet-4".to_string();
    }
    if short_name.contains("sonnet-3-7") {
        return "claude-sonnet-3-7".to_string();
    }
    if short_name.contains("haiku-4-5") {
        return "claude-haiku-4-5".to_string();
    }
    if short_name.contains("haiku-3-5") {
        return "claude-haiku-3-5".to_string();
    }
    // Unknown models get a generic name
    "claude".to_string()
}

/// Get the current client surface from environment.
pub fn get_client_surface() -> String {
    std::env::var(ai::CODE_ENTRYPOINT).unwrap_or_else(|_| "cli".to_string())
}

/// Build a surface key that includes the model name.
/// Format: "surface/model" (e.g., "cli/claude-sonnet")
pub fn build_surface_key(surface: &str, model: &str) -> String {
    format!("{}/{}", surface, model)
}

/// Compute SHA-256 hash of content.
pub fn compute_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Normalize file path to relative path from cwd for consistent tracking.
/// Resolves symlinks to handle /tmp vs /private/tmp on macOS.
pub fn normalize_file_path(file_path: &str) -> String {
    let cwd = get_attribution_repo_root();
    let cwd_path = Path::new(&cwd);
    let file_path_buf = PathBuf::from(file_path);

    if !file_path_buf.is_absolute() {
        return file_path.to_string();
    }

    // Resolve symlinks in both paths for consistent comparison
    // (e.g., /tmp -> /private/tmp on macOS)
    let resolved_path = std::fs::read_link(&file_path_buf)
        .map(|p| PathBuf::from(file_path).join(p))
        .unwrap_or_else(|_| file_path_buf.clone());

    let resolved_cwd = std::fs::read_link(cwd_path)
        .map(|p| cwd_path.join(p))
        .unwrap_or_else(|_| cwd_path.to_path_buf());

    let sep = std::path::MAIN_SEPARATOR;
    if resolved_path.starts_with(&resolved_cwd) || resolved_path == resolved_cwd {
        // Normalize to forward slashes so keys match git diff output on Windows
        return resolved_path
            .strip_prefix(&resolved_cwd)
            .map(|p| p.to_string_lossy().replace(sep, "/"))
            .unwrap_or_else(|_| file_path.to_string());
    }

    // Fallback: try original comparison
    if file_path.starts_with(&format!("{}{}", cwd, sep)) || file_path == cwd {
        return PathBuf::from(file_path)
            .strip_prefix(&cwd)
            .map(|p| p.to_string_lossy().replace(sep, "/"))
            .unwrap_or_else(|_| file_path.to_string());
    }

    file_path.to_string()
}

/// Expand a relative path to absolute path.
pub fn expand_file_path(file_path: &str) -> String {
    if Path::new(file_path).is_absolute() {
        file_path.to_string()
    } else {
        let repo_root = get_attribution_repo_root();
        Path::new(&repo_root)
            .join(file_path)
            .to_string_lossy()
            .to_string()
    }
}

/// Create an empty attribution state for a new session.
pub fn create_empty_attribution_state() -> AttributionState {
    AttributionState {
        file_states: HashMap::new(),
        session_baselines: HashMap::new(),
        surface: get_client_surface(),
        starting_head_sha: None,
        prompt_count: 0,
        prompt_count_at_last_commit: 0,
        permission_prompt_count: 0,
        permission_prompt_count_at_last_commit: 0,
        escape_count: 0,
        escape_count_at_last_commit: 0,
    }
}

/// Track a file modification by Claude.
/// Called after Edit/Write tool completes.
pub fn track_file_modification(
    state: AttributionState,
    file_path: &str,
    old_content: &str,
    new_content: &str,
    _user_modified: bool,
    mtime: Option<u64>,
) -> AttributionState {
    let normalized_path = normalize_file_path(file_path);
    let mtime = mtime.unwrap_or_else(current_timestamp);

    let new_file_state = compute_file_modification_state(
        &state.file_states,
        file_path,
        old_content,
        new_content,
        mtime,
    );

    if new_file_state.is_none() {
        return state;
    }

    let mut new_file_states = state.file_states.clone();
    new_file_states.insert(normalized_path, new_file_state.unwrap());

    AttributionState {
        file_states: new_file_states,
        ..state
    }
}

/// Track a file creation by Claude (e.g., via bash command).
/// Used when Claude creates a new file through a non-tracked mechanism.
pub fn track_file_creation(
    state: AttributionState,
    file_path: &str,
    content: &str,
    mtime: Option<u64>,
) -> AttributionState {
    // A creation is simply a modification from empty to the new content
    track_file_modification(state, file_path, "", content, false, mtime)
}

/// Track a file deletion by Claude (e.g., via bash rm command).
/// Used when Claude deletes a file through a non-tracked mechanism.
pub fn track_file_deletion(
    state: AttributionState,
    file_path: &str,
    old_content: &str,
) -> AttributionState {
    let normalized_path = normalize_file_path(file_path);
    let existing_state = state.file_states.get(&normalized_path);
    let existing_contribution = existing_state.map(|s| s.claude_contribution).unwrap_or(0);
    let deleted_chars = old_content.len() as u64;

    let new_file_state = FileAttributionState {
        content_hash: String::new(), // Empty hash for deleted files
        claude_contribution: existing_contribution + deleted_chars,
        mtime: current_timestamp(),
    };

    let mut new_file_states = state.file_states.clone();
    new_file_states.insert(normalized_path, new_file_state);

    AttributionState {
        file_states: new_file_states,
        ..state
    }
}

/// Track multiple file changes in bulk.
pub fn track_bulk_file_changes(
    state: AttributionState,
    changes: Vec<FileChange>,
) -> AttributionState {
    // Create ONE copy of the HashMap, then mutate it for each file
    let mut new_file_states = state.file_states.clone();

    for change in changes {
        let mtime = change.mtime.unwrap_or_else(current_timestamp);
        if change.change_type == FileChangeType::Deleted {
            let normalized_path = normalize_file_path(&change.path);
            let existing_state = new_file_states.get(&normalized_path);
            let existing_contribution = existing_state.map(|s| s.claude_contribution).unwrap_or(0);
            let deleted_chars = change.old_content.len() as u64;

            new_file_states.insert(
                normalized_path,
                FileAttributionState {
                    content_hash: String::new(),
                    claude_contribution: existing_contribution + deleted_chars,
                    mtime,
                },
            );
        } else {
            let new_file_state = compute_file_modification_state(
                &new_file_states,
                &change.path,
                &change.old_content,
                &change.new_content,
                mtime,
            );
            if let Some(file_state) = new_file_state {
                let normalized_path = normalize_file_path(&change.path);
                new_file_states.insert(normalized_path, file_state);
            }
        }
    }

    AttributionState {
        file_states: new_file_states,
        ..state
    }
}

/// File change for bulk tracking.
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub change_type: FileChangeType,
    pub old_content: String,
    pub new_content: String,
    pub mtime: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeType {
    Modified,
    Created,
    Deleted,
}

/// Convert attribution state to snapshot message for persistence.
pub fn state_to_snapshot_message(
    state: &AttributionState,
    message_id: &str,
) -> AttributionSnapshotMessage {
    AttributionSnapshotMessage {
        message_type: "attribution-snapshot".to_string(),
        message_id: message_id.to_string(),
        surface: state.surface.clone(),
        file_states: state.file_states.clone(),
        prompt_count: state.prompt_count,
        prompt_count_at_last_commit: state.prompt_count_at_last_commit,
        permission_prompt_count: state.permission_prompt_count,
        permission_prompt_count_at_last_commit: state.permission_prompt_count_at_last_commit,
        escape_count: state.escape_count,
        escape_count_at_last_commit: state.escape_count_at_last_commit,
    }
}

/// Restore attribution state from snapshot messages.
pub fn restore_attribution_state_from_snapshots(
    snapshots: &[AttributionSnapshotMessage],
) -> AttributionState {
    let mut state = create_empty_attribution_state();

    // Snapshots are full-state dumps, not deltas.
    // The last snapshot has the most recent count for every path.
    let Some(last_snapshot) = snapshots.last() else {
        return state;
    };

    state.surface = last_snapshot.surface.clone();
    state.file_states = last_snapshot.file_states.clone();

    // Restore prompt counts from the last snapshot (most recent state)
    state.prompt_count = last_snapshot.prompt_count;
    state.prompt_count_at_last_commit = last_snapshot.prompt_count_at_last_commit;
    state.permission_prompt_count = last_snapshot.permission_prompt_count;
    state.permission_prompt_count_at_last_commit =
        last_snapshot.permission_prompt_count_at_last_commit;
    state.escape_count = last_snapshot.escape_count;
    state.escape_count_at_last_commit = last_snapshot.escape_count_at_last_commit;

    state
}

/// Restore attribution state from log snapshots on session resume.
pub fn attribution_restore_state_from_log<F>(
    attribution_snapshots: Vec<AttributionSnapshotMessage>,
    on_update_state: F,
) where
    F: Fn(AttributionState),
{
    let state = restore_attribution_state_from_snapshots(&attribution_snapshots);
    on_update_state(state);
}

/// Increment promptCount and save an attribution snapshot.
/// Used to persist the prompt count across compaction.
pub fn increment_prompt_count(
    attribution: AttributionState,
    save_snapshot: impl Fn(AttributionSnapshotMessage),
) -> AttributionState {
    let new_attribution = AttributionState {
        prompt_count: attribution.prompt_count + 1,
        ..attribution
    };
    let snapshot = state_to_snapshot_message(&new_attribution, &uuid::Uuid::new_v4().to_string());
    save_snapshot(snapshot);
    new_attribution
}

// ============================================================================
// Private Functions
// ============================================================================

/// Compute the character contribution for a file modification.
/// Returns the FileAttributionState to store, or None if tracking failed.
fn compute_file_modification_state(
    existing_file_states: &HashMap<String, FileAttributionState>,
    file_path: &str,
    old_content: &str,
    new_content: &str,
    mtime: u64,
) -> Option<FileAttributionState> {
    let normalized_path = normalize_file_path(file_path);

    // Calculate Claude's character contribution
    let claude_contribution: u64;

    if old_content.is_empty() || new_content.is_empty() {
        // New file or full deletion - contribution is the content length
        claude_contribution = if old_content.is_empty() {
            new_content.len() as u64
        } else {
            old_content.len() as u64
        };
    } else {
        // Find actual changed region via common prefix/suffix matching.
        // This correctly handles same-length replacements (e.g., "Esc" -> "esc")
        // where Math.abs(newLen - oldLen) would be 0.
        let min_len = old_content.len().min(new_content.len());
        let mut prefix_end = 0;
        while prefix_end < min_len
            && old_content.as_bytes()[prefix_end] == new_content.as_bytes()[prefix_end]
        {
            prefix_end += 1;
        }

        let mut suffix_len = 0;
        while suffix_len < min_len - prefix_end
            && old_content.as_bytes()[old_content.len() - 1 - suffix_len]
                == new_content.as_bytes()[new_content.len() - 1 - suffix_len]
        {
            suffix_len += 1;
        }

        let old_changed_len = old_content.len() - prefix_end - suffix_len;
        let new_changed_len = new_content.len() - prefix_end - suffix_len;
        claude_contribution = old_changed_len.max(new_changed_len) as u64;
    }

    // Get current file state if it exists
    let existing_contribution = existing_file_states
        .get(&normalized_path)
        .map(|s| s.claude_contribution)
        .unwrap_or(0);

    Some(FileAttributionState {
        content_hash: compute_content_hash(new_content),
        claude_contribution: existing_contribution + claude_contribution,
        mtime,
    })
}

/// Get current timestamp in milliseconds since epoch.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================================
// Async Functions (placeholders - would need async runtime integration)
// ============================================================================

/// Check if the current repo is in the allowlist for internal model names.
/// This is a placeholder - would need proper async integration.
pub async fn is_internal_model_repo() -> bool {
    // Check cache first
    if let Some(class) = REPO_CLASS_CACHE.read().ok().and_then(|g| *g) {
        return class == RepoClass::Internal;
    }

    let cwd = get_attribution_repo_root();

    // TODO: Implement actual async check with get_remote_url_for_dir
    // For now, return false (safe default: don't leak)
    let _ = cwd;
    false
}

/// Get a file's modification time (mtimeMs), falling back to Date.now() if
/// the file doesn't exist.
pub async fn get_file_mtime(file_path: &str) -> u64 {
    let normalized_path = normalize_file_path(file_path);
    let abs_path = expand_file_path(&normalized_path);

    std::fs::metadata(&abs_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or_else(current_timestamp)
}

/// Calculate final attribution for staged files.
/// Compares session baseline to committed state.
pub async fn calculate_commit_attribution(
    states: Vec<AttributionState>,
    staged_files: Vec<String>,
) -> AttributionData {
    let cwd = get_attribution_repo_root();
    // TODO: Get actual session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    let mut files: HashMap<String, FileAttribution> = HashMap::new();
    let mut excluded_generated: Vec<String> = Vec::new();
    let mut surfaces: HashSet<String> = HashSet::new();
    let mut surface_counts: HashMap<String, u64> = HashMap::new();

    let mut total_claude_chars: u64 = 0;
    let mut total_human_chars: u64 = 0;

    // Merge file states from all sessions
    let mut merged_file_states: HashMap<String, FileAttributionState> = HashMap::new();
    let mut merged_baselines: HashMap<String, SessionBaseline> = HashMap::new();

    for state in &states {
        surfaces.insert(state.surface.clone());

        // Merge baselines (earliest baseline wins)
        for (path, baseline) in &state.session_baselines {
            if !merged_baselines.contains_key(path) {
                merged_baselines.insert(path.clone(), baseline.clone());
            }
        }

        // Merge file states (accumulate contributions)
        for (path, file_state) in &state.file_states {
            if let Some(existing) = merged_file_states.get(path) {
                merged_file_states.insert(
                    path.clone(),
                    FileAttributionState {
                        content_hash: file_state.content_hash.clone(),
                        claude_contribution: existing.claude_contribution
                            + file_state.claude_contribution,
                        mtime: file_state.mtime,
                    },
                );
            } else {
                merged_file_states.insert(path.clone(), file_state.clone());
            }
        }
    }

    // Process each staged file
    for file in staged_files {
        // Skip generated files (placeholder - would need is_generated_file)
        // if is_generated_file(&file) {
        //     excluded_generated.push(file.clone());
        //     continue;
        // }

        let abs_path = PathBuf::from(&cwd).join(&file);
        let file_state = merged_file_states.get(&file);
        let baseline = merged_baselines.get(&file);

        // Get the surface for this file
        let file_surface = states
            .first()
            .map(|s| s.surface.clone())
            .unwrap_or_else(get_client_surface);

        let (mut claude_chars, mut human_chars) = (0u64, 0u64);

        // Check if file was deleted (placeholder - would need is_file_deleted)
        // For now, check if file exists
        let deleted = !abs_path.exists();

        if deleted {
            // File was deleted
            if let Some(state) = file_state {
                claude_chars = state.claude_contribution;
                human_chars = 0;
            } else {
                // Human deleted this file - use diff size estimation
                human_chars = 100; // Minimum attribution for a deletion
            }
        } else {
            // File exists - use file size as proxy for char count
            if let Ok(stats) = std::fs::metadata(&abs_path) {
                if file_state.is_some() {
                    // We have tracked modifications for this file
                    claude_chars = file_state.map(|s| s.claude_contribution).unwrap_or(0);
                    human_chars = 0;
                } else if baseline.is_some() {
                    // File was modified but not tracked - human modification
                    human_chars = stats.len() as u64;
                } else {
                    // New file not created by Claude
                    human_chars = stats.len() as u64;
                }
            }
        }

        // Ensure non-negative values
        claude_chars = claude_chars.max(0);
        human_chars = human_chars.max(0);

        let total = claude_chars + human_chars;
        let percent = if total > 0 {
            ((claude_chars as f64 / total as f64) * 100.0).round() as u32
        } else {
            0
        };

        files.insert(
            file.clone(),
            FileAttribution {
                claude_chars,
                human_chars,
                percent,
                surface: file_surface.clone(),
            },
        );

        total_claude_chars += claude_chars;
        total_human_chars += human_chars;

        *surface_counts.entry(file_surface).or_insert(0) += claude_chars;
    }

    let total_chars = total_claude_chars + total_human_chars;
    let claude_percent = if total_chars > 0 {
        ((total_claude_chars as f64 / total_chars as f64) * 100.0).round() as u32
    } else {
        0
    };

    // Calculate surface breakdown (percentage of total content per surface)
    let mut surface_breakdown: HashMap<String, SurfaceBreakdown> = HashMap::new();
    for (surface, chars) in surface_counts {
        let percent = if total_chars > 0 {
            ((chars as f64 / total_chars as f64) * 100.0).round() as u32
        } else {
            0
        };
        surface_breakdown.insert(
            surface,
            SurfaceBreakdown {
                claude_chars: chars,
                percent,
            },
        );
    }

    AttributionData {
        version: 1,
        summary: AttributionSummary {
            claude_percent,
            claude_chars: total_claude_chars,
            human_chars: total_human_chars,
            surfaces: surfaces.into_iter().collect(),
        },
        files,
        surface_breakdown,
        excluded_generated,
        sessions: vec![session_id],
    }
}

/// Get staged files from git.
pub async fn get_staged_files() -> Vec<String> {
    // TODO: Implement with actual git command
    // For now, return empty
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_model_name() {
        assert_eq!(sanitize_model_name("opus-4-5-fast"), "claude-opus-4-5");
        assert_eq!(sanitize_model_name("sonnet-4"), "claude-sonnet-4");
        assert_eq!(sanitize_model_name("unknown"), "claude");
    }

    #[test]
    fn test_sanitize_surface_key() {
        assert_eq!(
            sanitize_surface_key("cli/opus-4-5-fast"),
            "cli/claude-opus-4-5"
        );
        assert_eq!(sanitize_surface_key("cli"), "cli");
    }

    #[test]
    fn test_compute_content_hash() {
        let hash1 = compute_content_hash("hello");
        let hash2 = compute_content_hash("hello");
        let hash3 = compute_content_hash("world");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_normalize_file_path() {
        // Test relative path stays relative
        assert_eq!(normalize_file_path("test.rs"), "test.rs");

        // Test absolute path normalization (depends on current dir)
        let abs_path = std::env::current_dir()
            .unwrap()
            .join("test.rs")
            .to_string_lossy()
            .to_string();
        let normalized = normalize_file_path(&abs_path);
        assert!(normalized.ends_with("test.rs") || normalized == abs_path);
    }

    #[test]
    fn test_create_empty_attribution_state() {
        let state = create_empty_attribution_state();
        assert!(state.file_states.is_empty());
        assert_eq!(state.prompt_count, 0);
    }

    #[test]
    fn test_track_file_creation() {
        let state = create_empty_attribution_state();
        let state = track_file_creation(state, "test.rs", "fn main() {}", None);
        assert!(state.file_states.contains_key("test.rs"));
    }

    #[test]
    fn test_track_file_modification() {
        let state = create_empty_attribution_state();
        let state = track_file_modification(state, "test.rs", "", "fn main() {}", false, None);
        assert!(state.file_states.contains_key("test.rs"));
    }
}
