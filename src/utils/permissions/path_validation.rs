// Source: ~/claudecode/openclaudecode/src/utils/permissions/pathValidation.ts
#![allow(dead_code)]

//! Path validation utilities for file system permission checks.
//!
//! Handles tilde expansion, glob pattern validation, and path resolution.

use std::path::{Path, MAIN_SEPARATOR};
use crate::types::permissions::{PermissionDecisionReason, ToolPermissionContext};
use super::filesystem::{
    expand_tilde, is_path_allowed, to_posix_path,
    get_glob_base_directory, contains_path_traversal,
    FileOperationType, PathCheckResult,
};

/// Maximum number of directories to list in error messages.
const MAX_DIRS_TO_LIST: usize = 5;

/// Regex for glob pattern characters.
const GLOB_PATTERN_CHARS: &str = "*?[]{}";

/// Checks if a path contains glob pattern characters.
fn has_glob_pattern(path: &str) -> bool {
    path.chars().any(|c| GLOB_PATTERN_CHARS.contains(c))
}

/// Checks if a resolved path is in the sandbox write allowlist.
pub fn is_path_in_sandbox_write_allowlist(_resolved_path: &str) -> bool {
    // Simplified — full implementation requires sandbox integration
    false
}

/// Validates a glob pattern by checking its base directory.
pub fn validate_glob_pattern(
    clean_path: &str,
    cwd: &str,
    tool_permission_context: &ToolPermissionContext,
    operation_type: FileOperationType,
) -> ResolvedPathCheckResult {
    if contains_path_traversal(clean_path) {
        let absolute_path = if Path::new(clean_path).is_absolute() {
            clean_path.to_string()
        } else {
            format!("{}/{}", cwd, clean_path)
        };
        let resolved_path = absolute_path.clone();
        let result = is_path_allowed(&resolved_path, tool_permission_context, operation_type, None);
        return ResolvedPathCheckResult {
            allowed: result.allowed,
            resolved_path,
            decision_reason: result.decision_reason,
        };
    }

    let base_path = get_glob_base_directory(clean_path);
    let absolute_base_path = if Path::new(&base_path).is_absolute() {
        base_path
    } else {
        format!("{}/{}", cwd, base_path)
    };
    let resolved_path = absolute_base_path.clone();
    let result = is_path_allowed(&resolved_path, tool_permission_context, operation_type, None);
    ResolvedPathCheckResult {
        allowed: result.allowed,
        resolved_path,
        decision_reason: result.decision_reason,
    }
}

/// Validates a file system path, handling tilde expansion and glob patterns.
pub fn validate_path(
    path: &str,
    cwd: &str,
    tool_permission_context: &ToolPermissionContext,
    operation_type: FileOperationType,
) -> ResolvedPathCheckResult {
    // Remove surrounding quotes if present
    let clean_path = expand_tilde(
        path.trim_start_matches(['\'', '"'])
            .trim_end_matches(['\'', '"']),
    );

    // Block UNC paths
    if contains_vulnerable_unc_path(&clean_path) {
        return ResolvedPathCheckResult {
            allowed: false,
            resolved_path: clean_path.clone(),
            decision_reason: Some(PermissionDecisionReason::Other {
                reason: "UNC network paths require manual approval".to_string(),
            }),
        };
    }

    // Reject tilde variants (~user, ~+, ~-, ~N)
    if clean_path.starts_with('~') {
        return ResolvedPathCheckResult {
            allowed: false,
            resolved_path: clean_path.clone(),
            decision_reason: Some(PermissionDecisionReason::Other {
                reason: "Tilde expansion variants (~user, ~+, ~-) in paths require manual approval".to_string(),
            }),
        };
    }

    // Reject shell expansion syntax
    if clean_path.contains('$') || clean_path.contains('%') || clean_path.starts_with('=') {
        return ResolvedPathCheckResult {
            allowed: false,
            resolved_path: clean_path.clone(),
            decision_reason: Some(PermissionDecisionReason::Other {
                reason: "Shell expansion syntax in paths requires manual approval".to_string(),
            }),
        };
    }

    // Handle glob patterns
    if has_glob_pattern(&clean_path) {
        if matches!(operation_type, FileOperationType::Write | FileOperationType::Create) {
            return ResolvedPathCheckResult {
                allowed: false,
                resolved_path: clean_path.clone(),
                decision_reason: Some(PermissionDecisionReason::Other {
                    reason: "Glob patterns are not allowed in write operations. Please specify an exact file path.".to_string(),
                }),
            };
        }
        return validate_glob_pattern(
            &clean_path,
            cwd,
            tool_permission_context,
            operation_type,
        );
    }

    // Resolve path
    let absolute_path = if Path::new(&clean_path).is_absolute() {
        clean_path
    } else {
        format!("{}/{}", cwd, clean_path)
    };
    let resolved_path = absolute_path.clone();

    let result = is_path_allowed(&resolved_path, tool_permission_context, operation_type, None);
    ResolvedPathCheckResult {
        allowed: result.allowed,
        resolved_path,
        decision_reason: result.decision_reason,
    }
}

/// Checks if a path contains a vulnerable UNC pattern.
pub fn contains_vulnerable_unc_path(path: &str) -> bool {
    // Simplified UNC detection
    path.starts_with("\\\\") && !path.starts_with("\\\\?\\")
        || path.starts_with("//") && !path.starts_with("//?/")
}

/// Resolved path check result.
pub struct ResolvedPathCheckResult {
    pub allowed: bool,
    pub resolved_path: String,
    pub decision_reason: Option<PermissionDecisionReason>,
}
