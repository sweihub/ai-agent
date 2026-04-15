// Source: /data/home/swei/claudecode/openclaudecode/src/utils/which.ts
//! `which` command implementation for finding executable paths.

use std::process::Command;

/// Async version of which - finds the full path to a command executable.
/// On Windows, uses `where.exe`. On POSIX systems, uses `which`.
pub async fn which(command: &str) -> Option<String> {
    which_impl(command).await
}

/// Sync version of which - finds the full path to a command executable.
pub fn which_sync(command: &str) -> Option<String> {
    which_impl_sync(command)
}

async fn which_impl(command: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, use where.exe and return the first result
        let output = Command::new("where.exe").arg(command).output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return None;
        }

        // where.exe returns multiple paths separated by newlines, return the first
        trimmed.split('\n').next().map(|s| s.trim().to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On POSIX systems (macOS, Linux, WSL), use which
        let output = Command::new("which").arg(command).output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return None;
        }

        Some(trimmed.to_string())
    }
}

fn which_impl_sync(command: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, use where.exe and return the first result
        let output = Command::new("where.exe").arg(command).output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return None;
        }

        // where.exe returns multiple paths separated by newlines, return the first
        trimmed.split('\n').next().map(|s| s.trim().to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On POSIX systems (macOS, Linux, WSL), use which
        let output = Command::new("which").arg(command).output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return None;
        }

        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_which_sync_existing_command() {
        // Test finding an existing command
        let result = which_sync("ls");
        assert!(result.is_some());
    }

    #[test]
    fn test_which_sync_nonexistent_command() {
        // Test finding a non-existent command
        let result = which_sync("nonexistent_command_xyz123");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_which_async_existing_command() {
        let result = which("ls").await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_which_async_nonexistent_command() {
        let result = which("nonexistent_command_xyz123").await;
        assert!(result.is_none());
    }
}
