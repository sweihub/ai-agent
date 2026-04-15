//! Memory directory path resolution - translated from memdir/paths.ts
//!
//! Handles finding and managing the memory directory for persistent storage.

use crate::constants::env::ai;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the default memory base directory
/// Resolution order:
///   1. AI_REMOTE_MEMORY_DIR env var (explicit override, set in CCR)
///   2. ~/.ai (default config home)
pub fn get_memory_base_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(ai::REMOTE_MEMORY_DIR) {
        return PathBuf::from(dir);
    }

    // Default to ~/.ai
    if let Some(home) = dirs::home_dir() {
        home.join(".ai")
    } else {
        // Fallback to current directory if no home found
        std::env::current_dir()
            .map(|p| p.join(".ai"))
            .unwrap_or_else(|_| PathBuf::from(".ai"))
    }
}

const AUTO_MEM_DIRNAME: &str = "memory";
const AUTO_MEM_ENTRYPOINT_NAME: &str = "MEMORY.md";

/// Validate and normalize a candidate auto-memory directory path.
///
/// SECURITY: Rejects paths that would be dangerous as a read-allowlist root
/// - relative: "../foo" — would be interpreted relative to CWD
/// - root/near-root (length < 3): "/" → "" after strip; "/a" too short
/// - Windows drive-root (C: regex): "C:\" → "C:" after strip
/// - UNC paths (\\server\share): network paths — opaque trust boundary
/// - null byte: can truncate in syscalls
///
/// Returns the normalized path with exactly one trailing separator,
/// or None if the path is unset/empty/rejected.
fn validate_memory_path(raw: Option<&str>, expand_tilde: bool) -> Option<PathBuf> {
    let raw = raw?;

    // Settings.json paths support ~/ expansion (user-friendly). The env var
    // override does not (it's set programmatically by Cowork/SDK, which should
    // always pass absolute paths). Bare "~", "~/", "~/.", "~/.." are NOT
    // expanded — they would make is_auto_mem_path() match all of $HOME or its
    // parent.
    let candidate = if expand_tilde && (raw.starts_with("~/") || raw.starts_with("~\\")) {
        let rest = &raw[2..];
        // Reject trivial remainders that would expand to $HOME or an ancestor
        // Using simple string checks instead of normalize()
        if rest.is_empty()
            || rest == "."
            || rest == ".."
            || rest.starts_with("../")
            || rest.starts_with("..\\")
        {
            return None;
        }
        if let Some(home) = dirs::home_dir() {
            home.join(rest)
        } else {
            return None;
        }
    } else {
        PathBuf::from(raw)
    };

    // Strip trailing separators
    let path_str = candidate.to_string_lossy().to_string();
    let normalized: String = path_str
        .chars()
        .rev()
        .skip_while(|c| *c == '/' || *c == '\\')
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    // Security checks
    if !Path::new(&normalized).is_absolute() {
        return None;
    }
    if normalized.len() < 3 {
        return None;
    }
    // Windows drive-root check (e.g., "C:")
    if normalized.chars().nth(1) == Some(':') && normalized.len() == 2 {
        return None;
    }
    // UNC paths
    if normalized.starts_with("\\\\") || normalized.starts_with("//") {
        return None;
    }
    // Null byte
    if normalized.contains('\0') {
        return None;
    }

    // Add exactly one trailing separator
    let sep = std::path::MAIN_SEPARATOR;
    if !normalized.ends_with(sep) {
        Some(format!("{}{}", normalized, sep).into())
    } else {
        Some(PathBuf::from(normalized))
    }
}

/// Direct override for the full auto-memory directory path via env var.
/// When set, get_auto_mem_path()/get_auto_mem_entrypoint() return this path directly
/// instead of computing `{base}/projects/{sanitized-cwd}/memory/`.
fn get_auto_mem_path_override() -> Option<PathBuf> {
    validate_memory_path(
        std::env::var(ai::COWORK_MEMORY_PATH_OVERRIDE)
            .ok()
            .as_deref(),
        false,
    )
}

/// Check if AI_COWORK_MEMORY_PATH_OVERRIDE is set to a valid override.
/// Use this as a signal that the SDK caller has explicitly opted into
/// the auto-memory mechanics — e.g. to decide whether to inject the
/// memory prompt when a custom system prompt replaces the default.
pub fn has_auto_mem_path_override() -> bool {
    get_auto_mem_path_override().is_some()
}

/// Get the project root - placeholder, would need integration with bootstrap/state
fn get_project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Get the canonical git repo root if available, otherwise falls back to
/// the stable project root. Uses find_canonical_git_root so all worktrees of the
/// same repo share one auto-memory directory.
fn get_auto_mem_base() -> PathBuf {
    // TODO: integrate with git utils to find canonical root
    // For now, fall back to project root
    get_project_root()
}

/// Get the auto-memory directory path.
///
/// Resolution order:
///   1. AI_COWORK_MEMORY_PATH_OVERRIDE env var (full-path override, used by Cowork)
///   2. autoMemoryDirectory in settings.json (trusted sources only: policy/local/user)
///   3. <memoryBase>/projects/<sanitized-git-root>/memory/
///      where memoryBase is resolved by get_memory_base_dir()
pub fn get_auto_mem_path() -> PathBuf {
    // Check for override first
    if let Some(r#override) = get_auto_mem_path_override() {
        return r#override;
    }

    // Check settings.json (simplified - full implementation would check policy/local/user settings)
    // TODO: integrate with settings system

    // Build path from base
    let base = get_memory_base_dir();
    let projects_dir = base.join("projects");
    let project_slug = sanitize_path_component(&get_auto_mem_base().to_string_lossy());
    let path = projects_dir.join(project_slug).join(AUTO_MEM_DIRNAME);

    // Ensure trailing separator
    let path_str = path.to_string_lossy().to_string();
    let sep = std::path::MAIN_SEPARATOR;
    if !path_str.ends_with(sep) {
        format!("{}{}", path_str, sep).into()
    } else {
        path
    }
}

/// Sanitize a string for use in path components
pub fn sanitize_path_component(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Returns the daily log file path for the given date (defaults to today).
/// Shape: <autoMemPath>/logs/YYYY/MM/YYYY-MM-DD.md
///
/// Used by assistant mode (KAIROS): rather than maintaining
/// MEMORY.md as a live index, the agent appends to a date-named log file
/// as it works. A separate nightly /dream skill distills these logs into
/// topic files + MEMORY.md.
pub fn get_auto_mem_daily_log_path(date: &std::time::SystemTime) -> PathBuf {
    let datetime: chrono::DateTime<chrono::Local> = (*date).into();
    let yyyy = datetime.format("%Y").to_string();
    let mm = datetime.format("%m").to_string();
    let dd = datetime.format("%d").to_string();
    get_auto_mem_path()
        .join("logs")
        .join(&yyyy)
        .join(&mm)
        .join(format!("{}-{}-{}.md", yyyy, mm, dd))
}

/// Returns the auto-memory entrypoint (MEMORY.md inside the auto-memory dir).
/// Follows the same resolution order as get_auto_mem_path().
pub fn get_auto_mem_entrypoint() -> PathBuf {
    get_auto_mem_path().join(AUTO_MEM_ENTRYPOINT_NAME)
}

/// Check if an absolute path is within the auto-memory directory.
///
/// SECURITY: Normalize to prevent path traversal bypasses via .. segments
pub fn is_auto_mem_path(absolute_path: &Path) -> bool {
    let mem_path = get_auto_mem_path();
    let path_str = absolute_path.to_string_lossy();
    let mem_path_str = mem_path.to_string_lossy().to_string();
    path_str.starts_with(mem_path_str.as_str())
}

/// Whether auto-memory features are enabled (memdir, agent memory, past session search).
/// Enabled by default. Priority chain (first defined wins):
///   1. AI_CODE_DISABLE_AUTO_MEMORY env var (1/true → OFF, 0/false → ON)
///   2. AI_CODE_SIMPLE (--bare) → OFF
///   3. CCR without persistent storage → OFF (no AI_CODE_REMOTE_MEMORY_DIR)
///   4. autoMemoryEnabled in settings.json (supports project-level opt-out)
///   5. Default: enabled
pub fn is_auto_memory_enabled() -> bool {
    // Check env var to disable
    if let Ok(env_val) = std::env::var(ai::CODE_DISABLE_AUTO_MEMORY) {
        if is_env_truthy(&env_val) {
            return false;
        }
        if is_env_defined_falsy(&env_val) {
            return true;
        }
    }

    // --bare / SIMPLE: prompts.ts already drops the memory section from the
    // system prompt via its SIMPLE early-return; this gate stops the other half
    if is_env_truthy(&std::env::var(ai::SIMPLE).unwrap_or_default()) {
        return false;
    }

    // CCR without persistent storage
    if is_env_truthy(&std::env::var(ai::REMOTE).unwrap_or_default())
        && std::env::var(ai::REMOTE_MEMORY_DIR).is_err()
    {
        return false;
    }

    // Check settings.json (simplified - would need full settings integration)
    // TODO: check getInitialSettings().autoMemoryEnabled

    // Default: enabled
    true
}

/// Check if an env var value is truthy (1, true, yes, on)
fn is_env_truthy(val: &str) -> bool {
    let lower = val.to_lowercase();
    lower == "1" || lower == "true" || lower == "yes" || lower == "on"
}

/// Check if an env var is defined but falsy (0, false, no, off)
fn is_env_defined_falsy(val: &str) -> bool {
    let lower = val.to_lowercase();
    lower == "0" || lower == "false" || lower == "no" || lower == "off"
}

/// Ensure a memory directory exists. Idempotent.
pub fn ensure_memory_dir_exists(memory_dir: &Path) -> std::io::Result<()> {
    match fs::create_dir_all(memory_dir) {
        Ok(_) => Ok(()),
        // EEXIST is handled by create_dir_all, but log other errors
        Err(e) => {
            // Log the error for debugging (in production, use proper logging)
            eprintln!("ensureMemoryDirExists failed for {:?}: {}", memory_dir, e);
            Ok(()) // Continue anyway - the model's Write will surface real perm errors
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path_component() {
        assert_eq!(sanitize_path_component("my-project"), "my-project");
        assert_eq!(sanitize_path_component("My Project!"), "My_Project_");
        assert_eq!(sanitize_path_component("123-test"), "123-test");
    }

    #[test]
    fn test_is_env_truthy() {
        assert!(is_env_truthy("1"));
        assert!(is_env_truthy("true"));
        assert!(is_env_truthy("yes"));
        assert!(is_env_truthy("on"));
        assert!(!is_env_truthy("0"));
        assert!(!is_env_truthy("false"));
    }

    #[test]
    fn test_get_auto_mem_path() {
        let path = get_auto_mem_path();
        assert!(path.is_absolute());
        assert!(
            path.to_string_lossy().ends_with("memory/")
                || path.to_string_lossy().ends_with("memory\\")
        );
    }

    #[test]
    fn test_is_auto_mem_path() {
        let mem_path = get_auto_mem_path();
        let inside = mem_path.join("test.md");
        let outside = PathBuf::from("/tmp/test.md");

        assert!(is_auto_mem_path(&inside));
        assert!(!is_auto_mem_path(&outside));
    }

    #[test]
    fn test_get_auto_mem_entrypoint() {
        let entrypoint = get_auto_mem_entrypoint();
        assert!(entrypoint.file_name().unwrap_or_default() == "MEMORY.md");
    }
}
