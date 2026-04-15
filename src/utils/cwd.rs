// Source: ~/claudecode/openclaudecode/src/utils/cwd.rs

use std::cell::RefCell;
use std::path::{Path, PathBuf};

thread_local! {
    /// Thread-local storage for cwd override.
    static CWD_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Run a function with an overridden working directory for the current thread.
/// All calls to pwd/get_cwd within the function will return the overridden cwd
/// instead of the global one.
pub fn run_with_cwd_override<T, F>(cwd: &Path, f: F) -> T
where
    F: FnOnce() -> T,
{
    CWD_OVERRIDE.with(|cell| {
        let prev = cell.borrow().clone();
        *cell.borrow_mut() = Some(cwd.to_path_buf());
        let result = f();
        *cell.borrow_mut() = prev;
        result
    })
}

/// Get the current working directory.
pub fn pwd() -> PathBuf {
    CWD_OVERRIDE
        .with(|cell| cell.borrow().clone())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")))
}

/// Get the current working directory or the original working directory if the current one is not available.
pub fn get_cwd() -> PathBuf {
    pwd()
}

/// Get the original working directory from environment.
pub fn get_original_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
}

/// Set the current working directory override.
pub fn set_cwd(path: &Path) {
    CWD_OVERRIDE.with(|cell| {
        *cell.borrow_mut() = Some(path.to_path_buf());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwd_returns_current_dir() {
        let cwd = pwd();
        assert!(cwd.is_absolute());
    }

    #[test]
    fn test_run_with_cwd_override() {
        let original = pwd();
        let test_dir = PathBuf::from("/tmp/test_cwd");

        run_with_cwd_override(&test_dir, || {
            assert_eq!(pwd(), test_dir);
        });

        // After the closure, cwd should be restored
        assert_eq!(pwd(), original);
    }
}
