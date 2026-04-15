//! PowerShell provider implementation.

use crate::constants::env::system;
use super::shell_provider::{ShellError, ShellExecCommand};
use super::shell_tool_utils::ShellType;
use std::collections::HashMap;

/// PowerShell invocation flags + command.
/// Shared by the provider's get_spawn_args and other paths.
pub fn build_powershell_args(cmd: &str) -> Vec<String> {
    vec![
        "-NoProfile".to_string(),
        "-NonInteractive".to_string(),
        "-Command".to_string(),
        cmd.to_string(),
    ]
}

/// Base64-encode a string as UTF-16LE for PowerShell's -EncodedCommand.
/// This encoding survives ANY shell-quoting layer.
fn encode_powershell_command(ps_command: &str) -> String {
    // Convert to UTF-16LE bytes
    let utf16: Vec<u8> = ps_command
        .encode_utf16()
        .flat_map(|c| c.to_le_bytes())
        .collect();

    // Base64 encode
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &utf16)
}

/// PowerShell shell provider implementation.
pub struct PowerShellProvider {
    shell_path: String,
    current_sandbox_tmp_dir: Option<String>,
}

impl PowerShellProvider {
    /// Create a new PowerShellProvider
    pub fn new(shell_path: &str) -> Self {
        Self {
            shell_path: shell_path.to_string(),
            current_sandbox_tmp_dir: None,
        }
    }

    /// Get shell type
    pub fn get_type(&self) -> ShellType {
        ShellType::PowerShell
    }

    /// Get shell path
    pub fn get_shell_path(&self) -> &str {
        &self.shell_path
    }

    /// Whether the shell is detached
    pub fn is_detached(&self) -> bool {
        false
    }

    /// Build the full command string including all PowerShell-specific setup.
    pub async fn build_exec_command(
        &mut self,
        command: &str,
        id: usize,
        sandbox_tmp_dir: Option<&str>,
        use_sandbox: bool,
    ) -> Result<ShellExecCommand, ShellError> {
        // Stash sandbox_tmp_dir for get_environment_overrides
        self.current_sandbox_tmp_dir = sandbox_tmp_dir.map(|s| s.to_string());

        let cwd_file_path = if use_sandbox && sandbox_tmp_dir.is_some() {
            format!("{}/claude-pwd-ps-{}", sandbox_tmp_dir.unwrap(), id)
        } else {
            let tmpdir = std::env::temp_dir();
            tmpdir
                .join(format!("claude-pwd-ps-{}", id))
                .to_string_lossy()
                .to_string()
        };

        let escaped_cwd_file_path = cwd_file_path.replace('\'', "''");

        // Exit-code capture: prefer $LASTEXITCODE when a native exe ran.
        // Fall back to $? for cmdlet-only pipelines.
        let cwd_tracking = format!(
            "\n; $_ec = if ($null -ne $LASTEXITCODE) {{ $LASTEXITCODE }} elseif ($?) {{ 0 }} else {{ 1 }}\n; (Get-Location).Path | Out-File -FilePath '{}' -Encoding utf8 -NoNewline\n; exit $_ec",
            escaped_cwd_file_path
        );

        let ps_command = format!("{}{}", command, cwd_tracking);

        // For sandbox path, build a command that invokes pwsh with the full flag set
        // For non-sandbox path, return the bare PS command
        let command_string = if use_sandbox {
            // Shell path is single-quoted, base64 encoded command
            let encoded = encode_powershell_command(&ps_command);
            format!(
                "'{}' -NoProfile -NonInteractive -EncodedCommand {}",
                self.shell_path.replace('\'', "'\\''"),
                encoded
            )
        } else {
            ps_command
        };

        Ok(ShellExecCommand {
            command_string,
            cwd_file_path,
        })
    }

    /// Get shell args for spawn
    pub fn get_spawn_args(&self, command_string: &str) -> Vec<String> {
        build_powershell_args(command_string)
    }

    /// Get extra env vars for this shell type
    pub async fn get_environment_overrides(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Apply session env vars set via /env
        // This would be implemented with session env vars integration
        // for (key, value) in get_session_env_vars() {
        //     env.insert(key, value);
        // }

        if let Some(ref tmpdir) = self.current_sandbox_tmp_dir {
            // PowerShell on Linux/macOS honors TMPDIR
            env.insert("TMPDIR".to_string(), tmpdir.clone());
            env.insert("AI_TMPDIR".to_string(), tmpdir.clone());
        }

        env
    }
}

impl Default for PowerShellProvider {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let shell = "powershell.exe".to_string();
        #[cfg(not(target_os = "windows"))]
        let shell = std::env::var(system::PATH)
            .ok()
            .and_then(|p| {
                p.split(':')
                    .find(|p| std::path::Path::new(&format!("{}/pwsh", p)).exists())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "pwsh".to_string());

        Self::new(&shell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_powershell_args() {
        let args = build_powershell_args("echo hello");
        assert_eq!(args[0], "-NoProfile");
        assert_eq!(args[1], "-NonInteractive");
        assert_eq!(args[2], "-Command");
        assert_eq!(args[3], "echo hello");
    }

    #[test]
    fn test_encode_powershell_command() {
        let encoded = encode_powershell_command("echo hello");
        // Base64 of "echo hello" in UTF-16LE
        assert!(!encoded.is_empty());
    }
}
