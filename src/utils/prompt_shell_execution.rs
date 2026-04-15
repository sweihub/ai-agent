//! Prompt shell execution utilities.

use std::process::Command;

/// Execute a shell command and return the result
pub async fn execute_prompt_shell(command: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .args(["-c", command])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Build a shell command with proper escaping
pub fn build_shell_command(program: &str, args: &[&str]) -> String {
    let mut cmd = program.to_string();

    for arg in args {
        cmd.push(' ');
        cmd.push_str(&shell_escape(arg));
    }

    cmd
}

/// Escape a string for shell usage
fn shell_escape(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}
