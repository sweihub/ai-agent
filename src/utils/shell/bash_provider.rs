//! Bash shell provider implementation.

use super::shell_provider::{ShellError, ShellExecCommand};
use super::shell_tool_utils::ShellType;
use crate::constants::env::{ai, system};
use std::collections::HashMap;

/// Returns a shell command to disable extended glob patterns for security.
/// Extended globs (bash extglob, zsh EXTENDED_GLOB) can be exploited via
/// malicious filenames that expand after our security validation.
fn get_disable_extglob_command(shell_path: &str) -> Option<String> {
    // When AI_SHELL_PREFIX is set, the wrapper may use a different shell
    // than shell_path, so we include both bash and zsh commands
    if std::env::var(ai::SHELL_PREFIX).is_ok() {
        // Redirect both stdout and stderr because zsh's command_not_found_handler
        // writes to stdout instead of stderr
        return Some(
            "{ shopt -u extglob || setopt NO_EXTENDED_GLOB; } >/dev/null 2>&1 || true".to_string(),
        );
    }

    // No shell prefix - use shell-specific command
    if shell_path.contains("bash") {
        Some("shopt -u extglob 2>/dev/null || true".to_string())
    } else if shell_path.contains("zsh") {
        Some("setopt NO_EXTENDED_GLOB 2>/dev/null || true".to_string())
    } else {
        // Unknown shell - do nothing
        None
    }
}

/// Bash shell provider implementation.
/// Provides command building and environment overrides for bash/zsh shells.
pub struct BashShellProvider {
    shell_path: String,
}

impl BashShellProvider {
    /// Create a new BashShellProvider
    pub fn new(shell_path: &str) -> Self {
        Self {
            shell_path: shell_path.to_string(),
        }
    }

    /// Get shell type
    pub fn get_type(&self) -> ShellType {
        ShellType::Bash
    }

    /// Get shell path
    pub fn get_shell_path(&self) -> &str {
        &self.shell_path
    }

    /// Whether the shell is detached
    pub fn is_detached(&self) -> bool {
        true
    }

    /// Build the full command string including all shell-specific setup.
    /// Includes: source snapshot, session env, disable extglob, eval-wrap, pwd tracking.
    pub async fn build_exec_command(
        &self,
        command: &str,
        id: usize,
        sandbox_tmp_dir: Option<&str>,
        use_sandbox: bool,
    ) -> Result<ShellExecCommand, ShellError> {
        let tmpdir = std::env::temp_dir();

        // shell_cwd_file_path: POSIX path used inside the bash command (pwd -P >| ...)
        // cwd_file_path: native OS path used by Node.js for readFile/unlink
        let (shell_cwd_file_path, cwd_file_path) = if use_sandbox {
            let sandbox = sandbox_tmp_dir.ok_or_else(|| {
                ShellError::BuildError(
                    "sandbox_tmp_dir required when use_sandbox is true".to_string(),
                )
            })?;
            (
                format!("{}/cwd-{}", sandbox, id),
                format!("{}/cwd-{}", sandbox, id),
            )
        } else {
            let cwd_file = format!("ai-{}-cwd", id);
            (
                tmpdir.join(&cwd_file).to_string_lossy().to_string(),
                tmpdir.join(&cwd_file).to_string_lossy().to_string(),
            )
        };

        // For now, we don't implement the snapshot feature in Rust
        // This would require integrating with bash initialization

        let mut command_parts: Vec<String> = Vec::new();

        // Source session environment variables captured from session start hooks
        // This would be implemented with session environment integration
        // let session_env_script = get_session_environment_script();
        // if let Some(script) = session_env_script {
        //     command_parts.push(script);
        // }

        // Disable extended glob patterns for security
        if let Some(disable_extglob_cmd) = get_disable_extglob_command(&self.shell_path) {
            command_parts.push(disable_extglob_cmd);
        }

        // Quote and wrap the command with eval
        // Use `pwd -P` to get the physical path of the current working directory
        command_parts.push(format!("eval {}", self.quote_command(command)));
        command_parts.push(format!(
            "pwd -P >| {}",
            self.quote_path(&shell_cwd_file_path)
        ));

        let command_string = command_parts.join(" && ");

        // Apply AI_SHELL_PREFIX if set
        let command_string = if let Ok(prefix) = std::env::var(ai::SHELL_PREFIX) {
            format!("{} -c '{}'", prefix, command_string)
        } else {
            command_string
        };

        Ok(ShellExecCommand {
            command_string,
            cwd_file_path,
        })
    }

    /// Get shell args for spawn
    pub fn get_spawn_args(&self, command_string: &str) -> Vec<String> {
        // For now, we always use login shell
        // In a full implementation, we'd check if snapshot file exists
        vec![
            "-c".to_string(),
            "-l".to_string(),
            command_string.to_string(),
        ]
    }

    /// Get extra env vars for this shell type
    pub async fn get_environment_overrides(&self, _command: &str) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Apply session env vars set via /env (child processes only, not the REPL)
        // This would be implemented with session env vars integration
        // for (key, value) in get_session_env_vars() {
        //     env.insert(key, value);
        // }

        env
    }

    /// Quote a command for safe shell execution
    fn quote_command(&self, command: &str) -> String {
        // Simple single-quote escaping for now
        // In production, would use proper shell quoting
        format!("'{}'", command.replace('\'', "'\\''"))
    }

    /// Quote a path for safe shell execution
    fn quote_path(&self, path: &str) -> String {
        self.quote_command(path)
    }
}

impl Default for BashShellProvider {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let shell = "bash".to_string();
        #[cfg(not(target_os = "windows"))]
        let shell = std::env::var(system::SHELL).unwrap_or_else(|_| "/bin/bash".to_string());

        Self::new(&shell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_exec_command() {
        let provider = BashShellProvider::new("/bin/bash");
        let result = provider
            .build_exec_command("echo hello", 1, None, false)
            .await;
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.command_string.contains("echo hello"));
    }

    #[test]
    fn test_get_spawn_args() {
        let provider = BashShellProvider::new("/bin/bash");
        let args = provider.get_spawn_args("echo hello");
        assert_eq!(args[0], "-c");
    }

    #[test]
    fn test_disable_extglob_command() {
        let cmd = get_disable_extglob_command("/bin/bash");
        assert!(cmd.is_some());

        let cmd = get_disable_extglob_command("/bin/zsh");
        assert!(cmd.is_some());

        let cmd = get_disable_extglob_command("/bin/sh");
        assert!(cmd.is_none());
    }
}
