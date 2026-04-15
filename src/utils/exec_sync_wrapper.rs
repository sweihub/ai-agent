// Source: ~/claudecode/openclaudecode/src/utils/execSyncWrapper.ts

use std::process::Command;
use std::time::Instant;

/// Threshold in milliseconds for logging a slow operation warning.
/// Operations taking longer than this are flagged as potential performance issues.
const SLOW_OPERATION_THRESHOLD_MS: u128 = 500;

/// Execute a command synchronously with slow operation logging.
///
/// Wrapped exec with slow operation logging.
/// Use this instead of std::process::Command::output directly to detect performance issues.
///
/// # Example
/// ```
/// use ai_agent::utils::exec_sync_wrapper::exec_sync_wrapper;
/// let result = exec_sync_wrapper("git", vec!["status".to_string()]);
/// ```
pub fn exec_sync_wrapper(command: &str, args: Vec<String>) -> Result<String, String> {
    let start = Instant::now();

    let output = Command::new(command)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute '{}': {}", command, e))?;

    let elapsed = start.elapsed().as_millis();

    if elapsed > SLOW_OPERATION_THRESHOLD_MS {
        // Truncate command for logging (first 100 chars, matching TS)
        let truncated = if command.len() > 100 {
            format!("{}...", &command[..100])
        } else {
            command.to_string()
        };
        log::warn!(
            "Slow operation detected: execSync: {} took {}ms",
            truncated,
            elapsed
        );
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let error_msg = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            format!(
                "Command exited with code {}: {}",
                output.status.code().unwrap_or(-1),
                stdout
            )
        } else {
            format!(
                "Command exited with code {}",
                output.status.code().unwrap_or(-1)
            )
        };
        return Err(error_msg);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_sync_wrapper_success() {
        let result = exec_sync_wrapper("echo", vec!["hello".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_exec_sync_wrapper_failure() {
        let result = exec_sync_wrapper("false", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_exec_sync_wrapper_nonexistent_command() {
        let result = exec_sync_wrapper("nonexistent_command_xyz", vec![]);
        assert!(result.is_err());
    }
}
