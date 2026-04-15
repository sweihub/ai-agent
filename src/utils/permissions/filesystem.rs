// Source: ~/claudecode/openclaudecode/src/utils/permissions/filesystem.ts
#![allow(dead_code)]

//! Filesystem-related permission utilities.
//!
//! Handles path validation, dangerous file detection, auto-edit safety checks,
//! and working directory permission validation.

use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use crate::types::permissions::{
    PermissionDecision, PermissionDecisionReason, PermissionRule,
    PermissionUpdate, PermissionUpdateDestination, ToolPermissionContext,
};

/// Dangerous files that should be protected from auto-editing.
/// These files can be used for code execution or data exfiltration.
pub const DANGEROUS_FILES: &[&str] = &[
    ".gitconfig",
    ".gitmodules",
    ".bashrc",
    ".bash_profile",
    ".zshrc",
    ".zprofile",
    ".profile",
    ".ripgreprc",
    ".mcp.json",
    ".claude.json",
];

/// Dangerous directories that should be protected from auto-editing.
pub const DANGEROUS_DIRECTORIES: &[&str] = &[
    ".git",
    ".vscode",
    ".idea",
    ".claude",
];

/// Normalizes a path for case-insensitive comparison.
/// Prevents bypassing security checks using mixed-case paths
/// on case-insensitive filesystems (macOS/Windows).
pub fn normalize_case_for_comparison(path: &str) -> String {
    path.to_lowercase()
}

/// If file_path is inside a .claude/skills/{name}/ directory (project or global),
/// return the skill name and a session-allow pattern scoped to just that skill.
pub fn get_claude_skill_scope(file_path: &str) -> Option<(String, String)> {
    let absolute_path = expand_path(file_path);
    let absolute_path_lower = normalize_case_for_comparison(&absolute_path);

    let cwd = std::env::current_dir().ok()?;
    let home = dirs::home_dir()?;

    let bases = [
        (
            cwd.join(".claude").join("skills"),
            "/.claude/skills/".to_string(),
        ),
        (
            home.join(".claude").join("skills"),
            "~/.claude/skills/".to_string(),
        ),
    ];

    for (dir, prefix) in &bases {
        let dir_lower = normalize_case_for_comparison(&dir.to_string_lossy());
        for sep_char in [MAIN_SEPARATOR, '/'] {
            let sep_lower = sep_char.to_lowercase().to_string();
            if absolute_path_lower.starts_with(&format!("{}{}", dir_lower, sep_lower)) {
                let dir_str = dir.to_string_lossy();
                let rest = &absolute_path[dir_str.len() + 1..];
                let slash = rest.find('/');
                let bslash = if MAIN_SEPARATOR == '\\' {
                    rest.find('\\')
                } else {
                    None
                };
                let cut = match (slash, bslash) {
                    (None, None) => return None,
                    (Some(s), None) => s,
                    (None, Some(b)) => b,
                    (Some(s), Some(b)) => s.min(b),
                };
                if cut == 0 {
                    return None;
                }
                let skill_name = &rest[..cut];
                if skill_name.is_empty() || skill_name == "." || skill_name.contains("..") {
                    return None;
                }
                // Reject glob metacharacters
                if skill_name.contains('*') || skill_name.contains('?') || skill_name.contains('[') || skill_name.contains(']') {
                    return None;
                }
                return Some((skill_name.to_string(), format!("{}{}/**", prefix, skill_name)));
            }
        }
    }

    None
}

/// Expands tilde (~) at the start of a path to the user's home directory.
pub fn expand_tilde(path: &str) -> String {
    if path == "~" || path.starts_with("~/") || (cfg!(windows) && path.starts_with("~\\")) {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.to_string_lossy(), &path[1..]);
        }
    }
    path.to_string()
}

/// Expands a path, resolving tilde and making absolute.
pub fn expand_path(path: &str) -> String {
    let expanded = expand_tilde(path);
    let p = Path::new(&expanded);
    if p.is_absolute() {
        p.to_string_lossy().to_string()
    } else {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(p).to_string_lossy().to_string())
            .unwrap_or(expanded)
    }
}

/// Converts a path to POSIX format for pattern matching.
pub fn to_posix_path(path: &str) -> String {
    if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.to_string()
    }
}

/// Calculates a relative path using POSIX separators.
pub fn relative_path(from: &str, to: &str) -> String {
    let from_path = Path::new(from);
    let to_path = Path::new(to);
    if let Ok(rel) = to_path.strip_prefix(from_path) {
        to_posix_path(&rel.to_string_lossy())
    } else {
        to.to_string()
    }
}

/// Checks if the file path is a Claude settings path.
pub fn is_claude_settings_path(file_path: &str) -> bool {
    let expanded = expand_path(file_path);
    let normalized = normalize_case_for_comparison(&expanded);
    let sep = MAIN_SEPARATOR.to_string();

    normalized.ends_with(&format!("{}{}claude{}settings.json", sep, sep, sep))
        || normalized.ends_with(&format!("{}{}claude{}settings.local.json", sep, sep, sep))
}

/// Checks if the file path is a Claude config file path.
pub fn is_claude_config_file_path(file_path: &str) -> bool {
    if is_claude_settings_path(file_path) {
        return true;
    }

    let cwd = std::env::current_dir().ok().unwrap_or_default();
    let commands_dir = cwd.join(".claude").join("commands");
    let agents_dir = cwd.join(".claude").join("agents");
    let skills_dir = cwd.join(".claude").join("skills");

    path_in_working_path(file_path, &commands_dir.to_string_lossy())
        || path_in_working_path(file_path, &agents_dir.to_string_lossy())
        || path_in_working_path(file_path, &skills_dir.to_string_lossy())
}

/// Checks if a path is within a working path.
pub fn path_in_working_path(path: &str, working_path: &str) -> bool {
    let absolute_path = expand_path(path);
    let absolute_working_path = expand_path(working_path);

    // Handle macOS symlink issues
    let normalized_path = absolute_path
        .replace("/private/var/", "/var/")
        .replace("/private/tmp/", "/tmp/")
        .replace("/private/tmp", "/tmp");
    let normalized_working_path = absolute_working_path
        .replace("/private/var/", "/var/")
        .replace("/private/tmp/", "/tmp/")
        .replace("/private/tmp", "/tmp");

    let case_normalized_path = normalize_case_for_comparison(&normalized_path);
    let case_normalized_working_path = normalize_case_for_comparison(&normalized_working_path);

    let relative = relative_path(&case_normalized_working_path, &case_normalized_path);
    if relative.is_empty() {
        return true;
    }

    if contains_path_traversal(&relative) {
        return false;
    }

    !Path::new(&relative).is_absolute()
}

/// Checks if a path contains traversal sequences.
pub fn contains_path_traversal(path: &str) -> bool {
    path.split(MAIN_SEPARATOR).any(|c| c == "..")
        || path.split('/').any(|c| c == "..")
        || path.split('\\').any(|c| c == "..")
}

/// Checks if a path has suspicious Windows patterns.
pub fn has_suspicious_windows_path_pattern(path: &str) -> bool {
    // Check for NTFS Alternate Data Streams (Windows/WSL only)
    if cfg!(windows) || std::env::var("WSL_DISTRO_NAME").is_ok() {
        let colon_index = path[2..].find(':');
        if colon_index.is_some() {
            return true;
        }
    }

    // Check for 8.3 short names
    if path.contains("~") {
        let re = regex::Regex::new(r"~\d").unwrap();
        if re.is_match(path) {
            return true;
        }
    }

    // Check for long path prefixes
    if path.starts_with(r"\\?\")
        || path.starts_with(r"\\.\")
        || path.starts_with("//?/")
        || path.starts_with("//./")
    {
        return true;
    }

    // Check for trailing dots and spaces
    if path.ends_with(|c: char| c == '.' || c.is_whitespace()) {
        return true;
    }

    // Check for DOS device names
    let dos_device_re = regex::Regex::new(r"\.(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])$").unwrap();
    if dos_device_re.is_match(path) {
        return true;
    }

    // Check for three or more consecutive dots as path component
    let dots_re = regex::Regex::new(r"(^|/|\\)\.{3,}(/|\\|$)").unwrap();
    if dots_re.is_match(path) {
        return true;
    }

    false
}

/// Checks if a file path is dangerous for auto-edit.
fn is_dangerous_file_path_to_auto_edit(path: &str) -> bool {
    let absolute_path = expand_path(path);
    let path_segments: Vec<&str> = absolute_path.split(MAIN_SEPARATOR).collect();
    let file_name = path_segments.last().copied().unwrap_or("");

    // Block UNC paths
    if path.starts_with("\\\\") || path.starts_with("//") {
        return true;
    }

    // Check dangerous directories
    for segment in &path_segments {
        let normalized_segment = normalize_case_for_comparison(segment);
        for dir in DANGEROUS_DIRECTORIES {
            if normalized_segment == normalize_case_for_comparison(dir) {
                // Special case: .claude/worktrees/ is not dangerous
                if *dir == ".claude" {
                    let idx = path_segments.iter().position(|&s| s == *segment).unwrap_or(0);
                    if idx + 1 < path_segments.len() {
                        let next = path_segments[idx + 1];
                        if normalize_case_for_comparison(next) == "worktrees" {
                            continue;
                        }
                    }
                }
                return true;
            }
        }
    }

    // Check dangerous files
    if !file_name.is_empty() {
        let normalized_file_name = normalize_case_for_comparison(file_name);
        if DANGEROUS_FILES.iter().any(|df| normalize_case_for_comparison(df) == normalized_file_name) {
            return true;
        }
    }

    false
}

/// Checks if a path is safe for auto-editing.
pub fn check_path_safety_for_auto_edit(
    path: &str,
    _precomputed_paths_to_check: Option<&[String]>,
) -> PathSafetyResult {
    let path_to_check = path.to_string();

    // Check for suspicious Windows path patterns
    if has_suspicious_windows_path_pattern(&path_to_check) {
        return PathSafetyResult::Unsafe {
            message: format!("Claude requested permissions to write to {}, which contains a suspicious Windows path pattern that requires manual approval.", path),
            classifier_approvable: false,
        };
    }

    // Check for Claude config files
    if is_claude_config_file_path(&path_to_check) {
        return PathSafetyResult::Unsafe {
            message: format!("Claude requested permissions to write to {}, but you haven't granted it yet.", path),
            classifier_approvable: true,
        };
    }

    // Check for dangerous files
    if is_dangerous_file_path_to_auto_edit(&path_to_check) {
        return PathSafetyResult::Unsafe {
            message: format!("Claude requested permissions to edit {} which is a sensitive file.", path),
            classifier_approvable: true,
        };
    }

    PathSafetyResult::Safe
}

/// Result of a path safety check.
pub enum PathSafetyResult {
    Safe,
    Unsafe {
        message: String,
        classifier_approvable: bool,
    },
}

/// Checks if a resolved path is dangerous for removal operations.
pub fn is_dangerous_removal_path(resolved_path: &str) -> bool {
    let forward_slashed = resolved_path.replace(&['\\', '/'][..], "/");

    if forward_slashed == "*" || forward_slashed.ends_with("/*") {
        return true;
    }

    let normalized_path = if forward_slashed == "/" {
        forward_slashed.clone()
    } else {
        forward_slashed.trim_end_matches('/').to_string()
    };

    if normalized_path == "/" {
        return true;
    }

    let drive_root_re = regex::Regex::new(r"^[A-Za-z]:/?$").unwrap();
    if drive_root_re.is_match(&normalized_path) {
        return true;
    }

    if let Some(home) = dirs::home_dir() {
        let normalized_home = home.to_string_lossy().replace('\\', "/");
        if normalized_path == normalized_home {
            return true;
        }
    }

    let parent = Path::new(&normalized_path).parent().map(|p| p.to_string_lossy().to_string());
    if parent.as_deref() == Some("/") {
        return true;
    }

    let drive_child_re = regex::Regex::new(r"^[A-Za-z]:/[^/]+$").unwrap();
    if drive_child_re.is_match(&normalized_path) {
        return true;
    }

    false
}

/// Validates a glob pattern by checking its base directory.
pub fn get_glob_base_directory(path: &str) -> String {
    let glob_pattern_re = regex::Regex::new(r"[*?\[\]{}]").unwrap();
    if let Some(m) = glob_pattern_re.find(path) {
        let before_glob = &path[..m.start()];
        let last_sep = before_glob.rfind('/');
        if let Some(idx) = last_sep {
            if idx == 0 {
                return "/".to_string();
            }
            return before_glob[..idx].to_string();
        }
        return ".".to_string();
    }
    path.to_string()
}

/// Checks if a resolved path is allowed for the given operation type.
pub fn is_path_allowed(
    resolved_path: &str,
    _context: &ToolPermissionContext,
    _operation_type: FileOperationType,
    _precomputed_paths_to_check: Option<&[String]>,
) -> PathCheckResult {
    // Simplified implementation — full implementation requires tool context integration
    PathCheckResult {
        allowed: false,
        decision_reason: None,
    }
}

/// Type of file operation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileOperationType {
    Read,
    Write,
    Create,
}

/// Result of a path check.
pub struct PathCheckResult {
    pub allowed: bool,
    pub decision_reason: Option<PermissionDecisionReason>,
}

/// Session memory directory path.
pub fn get_session_memory_dir() -> String {
    let project_dir = std::env::current_dir()
        .ok()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    format!("{}/session-memory/", project_dir)
}

/// Session memory file path.
pub fn get_session_memory_path() -> String {
    format!("{}summary.md", get_session_memory_dir())
}

/// Checks if path is within the session memory directory.
fn is_session_memory_path(absolute_path: &str) -> bool {
    let normalized = Path::new(absolute_path).to_string_lossy().to_string();
    normalized.starts_with(&get_session_memory_dir())
}

/// Checks if a path is an internal editable path.
pub fn check_editable_internal_path(
    _path: &str,
    _input: &serde_json::Value,
) -> InternalPathResult {
    InternalPathResult::Passthrough
}

/// Checks if a path is an internal readable path.
pub fn check_readable_internal_path(
    _path: &str,
    _input: &serde_json::Value,
) -> InternalPathResult {
    InternalPathResult::Passthrough
}

/// Result of internal path check.
pub enum InternalPathResult {
    Allow { decision_reason: PermissionDecisionReason },
    Passthrough,
}

/// Gets all working directories from context.
pub fn all_working_directories(context: &ToolPermissionContext) -> Vec<String> {
    let mut dirs = vec![std::env::current_dir()
        .ok()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()];
    dirs.extend(context.additional_working_directories.keys().cloned());
    dirs
}

/// Generates permission suggestions for a path.
pub fn generate_suggestions(
    _file_path: &str,
    _operation_type: &str,
    _tool_permission_context: &ToolPermissionContext,
    _paths_to_check: Option<&[String]>,
) -> Vec<PermissionUpdate> {
    vec![]
}

/// Matching rule for input path.
pub fn matching_rule_for_input(
    _path: &str,
    _tool_permission_context: &ToolPermissionContext,
    _tool_type: &str,
    _behavior: &str,
) -> Option<PermissionRule> {
    None
}

/// Checks if path is in allowed working path.
pub fn path_in_allowed_working_path(
    path: &str,
    tool_permission_context: &ToolPermissionContext,
    _precomputed_paths_to_check: Option<&[String]>,
) -> bool {
    let working_paths = all_working_directories(tool_permission_context);
    for working_path in &working_paths {
        if path_in_working_path(path, working_path) {
            return true;
        }
    }
    false
}

/// Formats directory list for display.
pub fn format_directory_list(directories: &[String]) -> String {
    const MAX_DIRS: usize = 5;
    let dir_count = directories.len();

    if dir_count <= MAX_DIRS {
        return directories
            .iter()
            .map(|d| format!("'{}'", d))
            .collect::<Vec<_>>()
            .join(", ");
    }

    let first_dirs = directories[..MAX_DIRS]
        .iter()
        .map(|d| format!("'{}'", d))
        .collect::<Vec<_>>()
        .join(", ");

    format!("{}, and {} more", first_dirs, dir_count - MAX_DIRS)
}
