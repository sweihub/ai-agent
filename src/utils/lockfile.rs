// Source: ~/claudecode/openclaudecode/src/utils/lockfile.ts

use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

/// Options for locking a file.
#[derive(Debug, Clone)]
pub struct LockOptions {
    /// How long to wait for the lock before failing (default: 10 seconds).
    pub timeout: Duration,
    /// How often to retry acquiring the lock (default: 100ms).
    pub retry_interval: Duration,
    /// Whether to stale-check an existing lock (default: false).
    pub stale: bool,
}

impl Default for LockOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            retry_interval: Duration::from_millis(100),
            stale: false,
        }
    }
}

/// A guard that releases the lock when dropped.
pub struct LockGuard {
    path: std::path::PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// State for the lockfile module.
struct LockfileState {
    _initialized: bool,
}

static LOCKFILE_STATE: OnceLock<LockfileState> = OnceLock::new();

fn get_lockfile_state() -> &'static LockfileState {
    LOCKFILE_STATE.get_or_init(|| LockfileState { _initialized: true })
}

/// Acquire a lock on a file asynchronously.
/// Returns a guard that releases the lock when dropped.
pub async fn lock(path: &Path, options: Option<LockOptions>) -> std::io::Result<LockGuard> {
    let _ = get_lockfile_state();
    let options = options.unwrap_or_default();
    let lock_path = path.with_extension("lock");

    let deadline = std::time::Instant::now() + options.timeout;

    while std::time::Instant::now() < deadline {
        // Try to create the lock file atomically
        match std::fs::File::create_new(&lock_path) {
            Ok(_) => {
                return Ok(LockGuard {
                    path: lock_path,
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Check for stale lock
                if options.stale {
                    if let Ok(metadata) = std::fs::metadata(&lock_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified.elapsed().unwrap_or(Duration::ZERO) > options.timeout {
                                // Stale lock, remove it
                                let _ = std::fs::remove_file(&lock_path);
                                continue;
                            }
                        }
                    }
                }
                tokio::time::sleep(options.retry_interval).await;
            }
            Err(e) => return Err(e),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!("Could not acquire lock on {:?}", path),
    ))
}

/// Acquire a lock on a file synchronously.
pub fn lock_sync(path: &Path, options: Option<LockOptions>) -> std::io::Result<LockGuard> {
    let _ = get_lockfile_state();
    let options = options.unwrap_or_default();
    let lock_path = path.with_extension("lock");

    let deadline = std::time::Instant::now() + options.timeout;

    while std::time::Instant::now() < deadline {
        match std::fs::File::create_new(&lock_path) {
            Ok(_) => {
                return Ok(LockGuard {
                    path: lock_path,
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if options.stale {
                    if let Ok(metadata) = std::fs::metadata(&lock_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified.elapsed().unwrap_or(Duration::ZERO) > options.timeout {
                                let _ = std::fs::remove_file(&lock_path);
                                continue;
                            }
                        }
                    }
                }
                std::thread::sleep(options.retry_interval);
            }
            Err(e) => return Err(e),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        format!("Could not acquire lock on {:?}", path),
    ))
}

/// Release a lock on a file.
pub fn unlock(path: &Path) -> std::io::Result<()> {
    let lock_path = path.with_extension("lock");
    std::fs::remove_file(lock_path)
}

/// Check if a file is currently locked.
pub fn check(path: &Path) -> bool {
    let lock_path = path.with_extension("lock");
    lock_path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_lock_and_unlock() {
        let dir = std::env::temp_dir();
        let test_file = dir.join("test_lockfile.txt");

        // Create test file
        let mut f = std::fs::File::create(&test_file).unwrap();
        writeln!(f, "test").unwrap();

        let guard = lock(&test_file, None).await.unwrap();
        assert!(check(&test_file));

        drop(guard);
        assert!(!check(&test_file));

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }
}
