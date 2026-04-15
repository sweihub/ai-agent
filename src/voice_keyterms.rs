//! Voice keyterms for improving STT accuracy in the voice_stream endpoint.
//!
//! Provides domain-specific vocabulary hints (Deepgram "keywords") so the STT
//! engine correctly recognises coding terminology, project names, and branch
//! names that would otherwise be misheard.

use std::collections::HashSet;
use std::path::Path;

use crate::bootstrap::state::get_project_root;
use crate::utils::git::get_branch;

// ─── Global keyterms ────────────────────────────────────────────────

/// Terms Deepgram consistently mangles without keyword hints.
/// Note: "Claude" and "Anthropic" are already server-side base keyterms.
/// Avoid terms nobody speaks aloud as-spelled (stdout → "standard out").
const GLOBAL_KEYTERMS: &[&str] = &[
    "MCP",
    "symlink",
    "grep",
    "regex",
    "localhost",
    "codebase",
    "TypeScript",
    "JSON",
    "OAuth",
    "webhook",
    "gRPC",
    "dotfiles",
    "subagent",
    "worktree",
];

// ─── Helpers ────────────────────────────────────────────────────────

/// Split an identifier (camelCase, PascalCase, kebab-case, snake_case, or
/// path segments) into individual words. Fragments of 2 chars or fewer are
/// discarded to avoid noise.
pub fn split_identifier(name: &str) -> Vec<String> {
    let mut result = name
        .replace(
            |c: char| c.is_lowercase() && c.is_ascii(),
            |c: char| {
                // Insert space before uppercase letters
                format!(" {}", c)
            },
        )
        .replace('-', " ")
        .replace('_', " ")
        .replace('.', " ")
        .replace('/', " ")
        .replace('\\', " ")
        .split_whitespace()
        .map(|w| w.trim().to_string())
        .filter(|w| w.len() > 2 && w.len() <= 20)
        .collect::<Vec<_>>();

    result.dedup();
    result
}

fn file_name_words(file_path: &str) -> Vec<String> {
    let stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    split_identifier(stem)
}

// ─── Public API ─────────────────────────────────────────────────────

const MAX_KEYTERMS: usize = 50;

/// Build a list of keyterms for the voice_stream STT endpoint.
///
/// Combines hardcoded global coding terms with session context (project name,
/// git branch, recent files) without any model calls.
pub async fn get_voice_keyterms(recent_files: Option<&HashSet<String>>) -> Vec<String> {
    let mut terms: HashSet<String> = GLOBAL_KEYTERMS.iter().map(|s| s.to_string()).collect();

    // Project root basename as a single term — users say "claude CLI internal"
    // as a phrase, not isolated words. Keeping the whole basename lets the
    // STT's keyterm boosting match the phrase regardless of separator.
    if let Ok(project_root) = get_project_root() {
        if let Some(name) = Path::new(&project_root)
            .file_name()
            .and_then(|n| n.to_str())
        {
            if name.len() > 2 && name.len() <= 50 {
                terms.insert(name.to_string());
            }
        }
    }

    // Git branch words (e.g. "feat/voice-keyterms" → "feat", "voice", "keyterms")
    if let Ok(branch) = get_branch() {
        if let Some(branch) = branch {
            for word in split_identifier(&branch) {
                terms.insert(word);
            }
        }
    }

    // Recent file names — only scan enough to fill remaining slots
    if let Some(files) = recent_files {
        for file_path in files {
            if terms.len() >= MAX_KEYTERMS {
                break;
            }
            for word in file_name_words(file_path) {
                terms.insert(word);
            }
        }
    }

    terms.into_iter().take(MAX_KEYTERMS).collect()
}
