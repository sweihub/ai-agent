// Source: /data/home/swei/claudecode/openclaudecode/src/utils/process.ts
//! Process management utilities.

use crate::constants::env::system;
use std::collections::HashMap;
use std::process::{Command, Stdio};

/// Get the current process ID
pub fn get_process_id() -> u32 {
    std::process::id()
}

/// Get process info as a map
pub fn get_process_info() -> HashMap<String, String> {
    let mut info = HashMap::new();
    info.insert("pid".to_string(), get_process_id().to_string());

    // Add environment info
    if let Ok(cwd) = std::env::current_dir() {
        info.insert("cwd".to_string(), cwd.to_string_lossy().to_string());
    }

    if let Ok(exe) = std::env::current_exe() {
        info.insert("exe".to_string(), exe.to_string_lossy().to_string());
    }

    info
}

/// Check if running in a specific environment
pub fn is_running_in_container() -> bool {
    // Check for common container indicators
    std::env::var(system::DOCKER_CONTAINER).is_ok()
        || std::env::var(system::KUBERNETES_SERVICE_HOST).is_ok()
        || std::path::Path::new("/.dockerenv").exists()
}

/// Get parent process ID
pub fn get_parent_process_id() -> Option<u32> {
    // On Unix, this would use std::os::unix::process::parent_id()
    // For cross-platform, we'd need a crate
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // This is a simplification; real implementation would need more work
        None
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// Run a command and get its output
pub fn run_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
