// Source: ~/claudecode/openclaudecode/src/utils/plugins/gitAvailability.ts

use std::sync::atomic::{AtomicBool, Ordering};

/// Check if a command is available in PATH.
///
/// Uses which to find the actual executable without executing it.
/// This is a security best practice to avoid executing arbitrary code
/// in untrusted directories.
async fn is_command_available(command: &str) -> bool {
    crate::utils::which(command).await.is_some()
}

/// Atomic flag for git availability cache.
static GIT_AVAILABLE: AtomicBool = AtomicBool::new(true);
static GIT_AVAILABLE_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Check if git is available on the system.
///
/// This is memoized so repeated calls within a session return the cached result.
/// Git availability is unlikely to change during a single CLI session.
///
/// Only checks PATH -- does not exec git. On macOS this means the /usr/bin/git
/// xcrun shim passes even without Xcode CLT installed; callers that hit
/// `xcrun: error:` at exec time should call mark_git_unavailable() so the rest
/// of the session behaves as though git is absent.
pub async fn check_git_available() -> bool {
    // Check if cache has been set
    if GIT_AVAILABLE_INITIALIZED.load(Ordering::SeqCst) {
        return GIT_AVAILABLE.load(Ordering::SeqCst);
    }

    let available = is_command_available("git").await;
    GIT_AVAILABLE.store(available, Ordering::SeqCst);
    GIT_AVAILABLE_INITIALIZED.store(true, Ordering::SeqCst);
    available
}

/// Force the memoized git-availability check to return false for the rest of
/// the session.
///
/// Call this when a git invocation fails in a way that indicates the binary
/// exists on PATH but cannot actually run -- the macOS xcrun shim being the
/// main case (`xcrun: error: invalid active developer path`). Subsequent
/// check_git_available() calls then short-circuit to false, so downstream code
/// that guards on git availability skips cleanly instead of failing repeatedly
/// with the same exec error.
pub fn mark_git_unavailable() {
    GIT_AVAILABLE.store(false, Ordering::SeqCst);
    GIT_AVAILABLE_INITIALIZED.store(true, Ordering::SeqCst);
}

/// Clear the git availability cache.
/// Used for testing purposes.
pub fn clear_git_availability_cache() {
    GIT_AVAILABLE.store(true, Ordering::SeqCst);
    GIT_AVAILABLE_INITIALIZED.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clear_cache() {
        clear_git_availability_cache();
        assert!(!GIT_AVAILABLE_INITIALIZED.load(Ordering::SeqCst));
    }

    #[test]
    fn test_mark_unavailable() {
        mark_git_unavailable();
        assert!(!GIT_AVAILABLE.load(Ordering::SeqCst));
        assert!(GIT_AVAILABLE_INITIALIZED.load(Ordering::SeqCst));
        clear_git_availability_cache();
    }
}
