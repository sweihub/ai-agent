// Source: ~/claudecode/openclaudecode/src/utils/promptShellExecution.ts
//! Skill prompt shell command execution.
//!
//! Parses skill markdown content and executes embedded shell commands.
//! Supports two syntaxes:
//! - Code blocks: ```! command ```
//! - Inline: !`command`
//!
//! Results are substituted back into the prompt text.

use crate::error::AgentError;
use futures_util::future::join_all;
use log::warn;
use regex::Regex;
use std::process::Command;
use tokio::time::timeout;

/// Regex for code block shell commands: ```! command ```
fn block_pattern() -> &'static Regex {
    lazy_static::lazy_static! {
        static ref BLOCK: Regex = Regex::new(r"```\!\s*\n?([\s\S]*?)\n?```").unwrap();
    }
    &BLOCK
}

/// Regex for inline shell commands: !`command`
/// Requires whitespace or start-of-line before ! to prevent false matches.
/// Uses (^|\s) capture group instead of lookbehind (Rust regex requires fixed-width).
fn inline_pattern() -> &'static Regex {
    lazy_static::lazy_static! {
        static ref INLINE: Regex = Regex::new(r"(^|\s)!`([^`]+)`").unwrap();
    }
    &INLINE
}

/// Shell type from skill frontmatter
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FrontmatterShell {
    #[default]
    Bash,
    PowerShell,
}

impl FrontmatterShell {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "powershell" => FrontmatterShell::PowerShell,
            _ => FrontmatterShell::Bash,
        }
    }
}

/// Result of executing a single shell command
struct ShellOutput {
    stdout: String,
    stderr: String,
}

/// Format shell output for inline or block context
fn format_shell_output(stdout: &str, stderr: &str, inline: bool) -> String {
    let mut parts = Vec::new();

    if !stdout.trim().is_empty() {
        parts.push(stdout.trim().to_string());
    }

    if !stderr.trim().is_empty() {
        if inline {
            parts.push(format!("[stderr: {}]", stderr.trim()));
        } else {
            parts.push(format!("[stderr]\n{}", stderr.trim()));
        }
    }

    if inline {
        parts.join(" ")
    } else {
        parts.join("\n")
    }
}

/// Execute a single shell command, returning output
async fn execute_single_command(
    command: String,
    shell_bin: String,
    shell_arg: String,
    _tool_name: String,
) -> Result<ShellOutput, String> {
    let result = timeout(
        std::time::Duration::from_secs(30),
        tokio::task::spawn_blocking(move || {
            let output = Command::new(&shell_bin)
                .args([&shell_arg, &command])
                .output()
                .map_err(|e| format!("Failed to spawn shell: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                return Err(format!(
                    "Shell command failed (exit {}): {}",
                    output.status,
                    if !stderr.is_empty() { stderr } else { stdout }
                ));
            }

            Ok(ShellOutput {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }),
    )
    .await;

    match result {
        Ok(Ok(Ok(output))) => Ok(output),
        Ok(Ok(Err(e))) => Err(e),
        Ok(Err(join_err)) => Err(format!("Shell task failed: {}", join_err)),
        Err(_) => Err("Shell command timed out (30s)".to_string()),
    }
}

/// Resolve the shell binary and argument based on the requested `FrontmatterShell`.
///
/// When `FrontmatterShell::PowerShell` is requested, attempts to find `pwsh`.
/// Falls back to `bash -c` with a warning log if `pwsh` is not available.
fn resolve_shell_tool(shell: &FrontmatterShell) -> (String, String, String) {
    match shell {
        FrontmatterShell::Bash => ("bash".to_string(), "-c".to_string(), "Bash".to_string()),
        FrontmatterShell::PowerShell => {
            if which_command("pwsh").is_some() {
                ("pwsh".to_string(), "-c".to_string(), "PowerShell".to_string())
            } else {
                warn!(
                    "PowerShell shell requested but 'pwsh' is not available, falling back to bash"
                );
                ("bash".to_string(), "-c".to_string(), "Bash".to_string())
            }
        }
    }
}

/// Internal: resolve shell with an optional custom PATH override (for testing)
#[allow(dead_code)]
fn resolve_shell_tool_with_path(
    shell: &FrontmatterShell,
    path_override: &std::ffi::OsStr,
) -> (String, String, String) {
    match shell {
        FrontmatterShell::Bash => ("bash".to_string(), "-c".to_string(), "Bash".to_string()),
        FrontmatterShell::PowerShell => {
            if which_command_in_path("pwsh", path_override).is_some() {
                ("pwsh".to_string(), "-c".to_string(), "PowerShell".to_string())
            } else {
                ("bash".to_string(), "-c".to_string(), "Bash".to_string())
            }
        }
    }
}

/// Check if a command exists in PATH (cross-platform helper for resolving `pwsh`)
fn which_command(cmd: &str) -> Option<std::path::PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let full = dir.join(cmd);
        if full.is_file() {
            return Some(full);
        }
    }
    None
}

/// Internal: check if a command exists in a specific PATH string (for testing)
#[allow(dead_code)]
fn which_command_in_path(cmd: &str, path_env: &std::ffi::OsStr) -> Option<std::path::PathBuf> {
    for dir in std::env::split_paths(path_env) {
        let full = dir.join(cmd);
        if full.is_file() {
            return Some(full);
        }
    }
    None
}

/// Parse shell commands from text and execute them, substituting output back.
///
/// Scans for both block (` ```! ```) and inline (`!```) patterns.
/// Commands are executed in parallel. On failure, the error is substituted
/// back into the text in place of the command.
///
/// The optional `can_execute` callback is invoked before each command to check
/// permissions. If it returns `false`, the command is skipped and `[Permission
/// denied]` is substituted instead. The callback receives `(command, tool_name)`
/// where `tool_name` is "Bash" or "PowerShell".
pub async fn execute_shell_commands_in_prompt<F>(
    text: &str,
    shell: &FrontmatterShell,
    skill_name: &str,
    can_execute: Option<&F>,
) -> String
where
    F: Fn(&str, &str) -> bool + Send + Sync + ?Sized,
{
    // Collect all matches with their positions and types
    let mut matches: Vec<(usize, usize, String, bool)> = Vec::new();

    for cap in block_pattern().captures_iter(text) {
        if let Some(full) = cap.get(0) {
            matches.push((full.start(), full.end(), full.as_str().to_string(), false));
        }
    }

    if text.contains("!`") {
        for cap in inline_pattern().captures_iter(text) {
            if let (Some(full), Some(prefix)) = (cap.get(0), cap.get(1)) {
                // Start from the `!` character, not the whitespace prefix
                let pattern_start = prefix.end();
                let pattern = text[pattern_start..full.end()].to_string();
                matches.push((pattern_start, full.end(), pattern, true));
            }
        }
    }

    if matches.is_empty() {
        return text.to_string();
    }

    // Build command list
    let commands: Vec<(String, String, bool)> = matches
        .iter()
        .map(|(_, _, pattern, inline)| {
            let command = if *inline {
                // Pattern is !`command` - extract between !` and `
                if let Some(stripped) = pattern.strip_prefix("!`") {
                    stripped.strip_suffix('`')
                        .map(|s| s.trim().to_string())
                        .unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                block_pattern()
                    .captures(pattern)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default()
            };
            (pattern.clone(), command, *inline)
        })
        .collect();

    // Resolve shell binary
    let (shell_bin, shell_arg, tool_name) = resolve_shell_tool(shell);

    // Execute all commands in parallel
    let futures: Vec<_> = commands
        .into_iter()
        .map(|(pattern, command, inline)| {
            let shell_bin = shell_bin.to_string();
            let shell_arg = shell_arg.to_string();
            let tool_name = tool_name.to_string();
            let skill_name = skill_name.to_string();
            async move {
                if command.is_empty() {
                    return (pattern.clone(), pattern);
                }

                // Permission check
                if let Some(ref cb) = can_execute {
                    if !cb(&command, &tool_name) {
                        warn!(
                            "Shell command permission denied in skill '{}': {}",
                            skill_name, command
                        );
                        return (pattern.clone(), "[Permission denied]".to_string());
                    }
                }

                match execute_single_command(command, shell_bin, shell_arg, tool_name).await {
                    Ok(output) => {
                        let formatted =
                            format_shell_output(&output.stdout, &output.stderr, inline);
                        (pattern.clone(), formatted)
                    }
                    Err(e) => {
                        let error_msg = if inline {
                            format!("[Error: {}]", e)
                        } else {
                            format!("[Error]\n{}", e)
                        };
                        (pattern.clone(), error_msg)
                    }
                }
            }
        })
        .collect();

    let mut results: Vec<(String, String)> = join_all(futures).await;

    // Build result by replacing matches in reverse order to preserve positions
    let mut result = text.to_string();
    for (start, end, pattern, _) in matches.iter().rev() {
        if let Some(pos) = results.iter().position(|(p, _)| p == pattern) {
            let (_, replacement) = results.remove(pos);
            result.replace_range(*start..*end, &replacement);
        }
    }

    result
}

// ============================================================================
// Legacy helpers (kept for backwards compatibility)
// ============================================================================

/// Execute a shell command and return the result (legacy API)
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

/// Build a shell command with proper escaping (legacy API)
pub fn build_shell_command(program: &str, args: &[&str]) -> String {
    let mut cmd = program.to_string();
    for arg in args {
        cmd.push(' ');
        cmd.push_str(&shell_escape(arg));
    }
    cmd
}

fn shell_escape(s: &str) -> String {
    if s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

/// Check if a shell command in a skill should be allowed.
///
/// This is a synchronous pre-check. The actual permission gating
/// happens in `execute_shell_commands_in_prompt` via the `can_execute` callback.
/// This function validates that the tool name is recognized.
pub fn can_execute_skill_shell(_command: &str, tool_name: &str) -> Result<(), AgentError> {
    match tool_name {
        "Bash" | "bash" | "PowerShell" | "powershell" => Ok(()),
        _ => Err(AgentError::Tool(format!("Unsupported shell tool: {}", tool_name))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_pattern_matches() {
        let text = "```!\necho hello\n```";
        assert!(block_pattern().is_match(text));
        let cap = block_pattern().captures(text).unwrap();
        assert!(cap.get(1).is_some());
    }

    #[test]
    fn test_block_pattern_multiline() {
        let text = "```!\necho hello\necho world\n```";
        let cap = block_pattern().captures(text).unwrap();
        let cmd = cap.get(1).unwrap().as_str().trim();
        assert_eq!(cmd, "echo hello\necho world");
    }

    #[test]
    fn test_inline_pattern_matches() {
        assert!(inline_pattern().is_match("Run !`ls` to see files"));
    }

    #[test]
    fn test_inline_pattern_no_match_without_whitespace() {
        assert!(!inline_pattern().is_match("x!`this`"));
    }

    #[test]
    fn test_inline_pattern_extract_command() {
        let cap = inline_pattern().captures("Run !`echo hi` now").unwrap();
        assert_eq!(cap.get(2).unwrap().as_str(), "echo hi");
    }

    #[test]
    fn test_format_shell_output_stdout_only() {
        assert_eq!(format_shell_output("hello world", "", false), "hello world");
    }

    #[test]
    fn test_format_shell_output_with_stderr_block() {
        assert_eq!(
            format_shell_output("stdout", "stderr msg", false),
            "stdout\n[stderr]\nstderr msg"
        );
    }

    #[test]
    fn test_format_shell_output_with_stderr_inline() {
        assert_eq!(
            format_shell_output("stdout", "stderr msg", true),
            "stdout [stderr: stderr msg]"
        );
    }

    #[test]
    fn test_format_shell_output_empty() {
        assert_eq!(format_shell_output("", "", false), "");
    }

    #[tokio::test]
    async fn test_execute_block_command() {
        let result = execute_shell_commands_in_prompt(
            "Before ```!\necho hello\n``` After",
            &FrontmatterShell::Bash,
            "test-skill",
            None::<&(dyn Fn(&str, &str) -> bool + Send + Sync)>,
        )
        .await;
        assert!(result.contains("hello"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
        assert!(!result.contains("```!"));
    }

    #[tokio::test]
    async fn test_execute_inline_command() {
        let result = execute_shell_commands_in_prompt(
            "Count: !`echo 42` items",
            &FrontmatterShell::Bash,
            "test-skill",
            None::<&(dyn Fn(&str, &str) -> bool + Send + Sync)>,
        )
        .await;
        assert!(result.contains("42"));
        assert!(!result.contains("!`echo 42`"));
    }

    #[tokio::test]
    async fn test_no_shell_commands() {
        let text = "This is plain text with no commands";
        let result = execute_shell_commands_in_prompt(
            text,
            &FrontmatterShell::Bash,
            "test",
            None::<&(dyn Fn(&str, &str) -> bool + Send + Sync)>,
        )
        .await;
        assert_eq!(result, text);
    }

    #[tokio::test]
    async fn test_failed_command_substitutes_error() {
        let result =
            execute_shell_commands_in_prompt("```!\nexit 1\n```", &FrontmatterShell::Bash, "test", None::<&(dyn Fn(&str, &str) -> bool + Send + Sync)>)
                .await;
        assert!(result.contains("[Error]"));
        assert!(!result.contains("```!"));
    }

    #[tokio::test]
    async fn test_multiple_commands() {
        let result = execute_shell_commands_in_prompt(
            "A ```!\necho one\n``` B !`echo two` C",
            &FrontmatterShell::Bash,
            "test-skill",
            None::<&(dyn Fn(&str, &str) -> bool + Send + Sync)>,
        )
        .await;
        assert!(result.contains("one"));
        assert!(result.contains("two"));
        assert!(result.contains("A"));
        assert!(result.contains("B"));
        assert!(result.contains("C"));
    }

    #[tokio::test]
    async fn test_command_with_stderr() {
        let result = execute_shell_commands_in_prompt(
            "```!\necho out && echo err >&2\n```",
            &FrontmatterShell::Bash,
            "test-skill",
            None::<&(dyn Fn(&str, &str) -> bool + Send + Sync)>,
        )
        .await;
        assert!(result.contains("out"));
        assert!(result.contains("err") || result.contains("[stderr]"));
    }

    #[test]
    fn test_frontmatter_shell_from_str() {
        assert_eq!(FrontmatterShell::from_str("bash"), FrontmatterShell::Bash);
        assert_eq!(
            FrontmatterShell::from_str("powershell"),
            FrontmatterShell::PowerShell
        );
        assert_eq!(FrontmatterShell::from_str("unknown"), FrontmatterShell::Bash);
        assert_eq!(FrontmatterShell::from_str(""), FrontmatterShell::Bash);
    }

    #[test]
    fn test_shell_escape_safe() {
        assert_eq!(shell_escape("hello"), "hello");
    }

    #[test]
    fn test_shell_escape_needs_quotes() {
        // "he'llo" -> replace ' -> '\\'' -> he'\\''llo -> 'he'\\''llo'
        assert_eq!(shell_escape("he'llo"), "'he'\\''llo'");
    }

    #[test]
    fn test_build_shell_command() {
        assert_eq!(build_shell_command("echo", &["hello", "world"]), "echo hello world");
    }

    #[tokio::test]
    async fn test_execute_prompt_shell() {
        let result = execute_prompt_shell("echo -n test").await;
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_can_execute_skill_shell() {
        assert!(can_execute_skill_shell("echo hello", "Bash").is_ok());
    }

    #[test]
    fn test_can_execute_skill_shell_unsupported_tool() {
        assert!(can_execute_skill_shell("echo hello", "Fish").is_err());
    }

    #[test]
    fn test_can_execute_skill_shell_powershell() {
        assert!(can_execute_skill_shell("Write-Host hello", "PowerShell").is_ok());
    }

    /// Test that denied commands show [Permission denied]
    #[tokio::test]
    async fn test_permission_denied_substitutes_message() {
        // Callback that denies all commands
        let deny_all = |_cmd: &str, _tool: &str| false;
        let result = execute_shell_commands_in_prompt(
            "Before ```!\necho hello\n``` After",
            &FrontmatterShell::Bash,
            "test-skill",
            Some(&deny_all),
        )
        .await;
        assert!(result.contains("[Permission denied]"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
        assert!(!result.contains("hello"));
    }

    /// Test that denied inline commands show [Permission denied]
    #[tokio::test]
    async fn test_permission_denied_inline_substitutes_message() {
        let deny_all = |_cmd: &str, _tool: &str| false;
        let result = execute_shell_commands_in_prompt(
            "Count: !`echo 42` items",
            &FrontmatterShell::Bash,
            "test-skill",
            Some(&deny_all),
        )
        .await;
        assert!(result.contains("[Permission denied]"));
        assert!(!result.contains("42"));
        assert!(!result.contains("!`echo 42`"));
    }

    /// Test that allowed commands run normally with can_execute callback
    #[tokio::test]
    async fn test_permission_allowed_executes() {
        let allow_all = |_cmd: &str, _tool: &str| true;
        let result = execute_shell_commands_in_prompt(
            "Before ```!\necho hello\n``` After",
            &FrontmatterShell::Bash,
            "test-skill",
            Some(&allow_all),
        )
        .await;
        assert!(result.contains("hello"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
        assert!(!result.contains("[Permission denied]"));
    }

    /// Test that allowed inline commands run normally
    #[tokio::test]
    async fn test_permission_allowed_inline_executes() {
        let allow_all = |_cmd: &str, _tool: &str| true;
        let result = execute_shell_commands_in_prompt(
            "Count: !`echo 42` items",
            &FrontmatterShell::Bash,
            "test-skill",
            Some(&allow_all),
        )
        .await;
        assert!(result.contains("42"));
        assert!(!result.contains("[Permission denied]"));
    }

    /// Test selective allow/deny: allow only "echo" commands
    #[tokio::test]
    async fn test_permission_selective() {
        let selective = |cmd: &str, _tool: &str| cmd.starts_with("echo");
        let result = execute_shell_commands_in_prompt(
            "A ```!\necho one\n``` B ```!\nexit 1\n```",
            &FrontmatterShell::Bash,
            "test-skill",
            Some(&selective),
        )
        .await;
        assert!(result.contains("one"));
        assert!(result.contains("[Permission denied]"));
    }

    /// Test that PowerShell falls back to bash when pwsh is not available.
    /// Uses a synthetic PATH that contains no pwsh binary.
    #[test]
    fn test_powershell_fallback_to_bash() {
        // Use a PATH that definitely has no pwsh
        let fake_path = std::ffi::OsStr::new("/nonexistent/path");
        let (bin, arg, tool) =
            resolve_shell_tool_with_path(&FrontmatterShell::PowerShell, fake_path);
        assert_eq!(bin, "bash");
        assert_eq!(arg, "-c");
        assert_eq!(tool, "Bash");
    }

    /// Test that PowerShell resolves to pwsh when available
    #[test]
    fn test_powershell_resolves_when_pwsh_available() {
        let current_path = std::env::var_os("PATH");
        if let Some(ref p) = current_path {
            let (bin, _arg, tool) =
                resolve_shell_tool_with_path(&FrontmatterShell::PowerShell, p.as_ref());
            if which_command_in_path("pwsh", p.as_ref()).is_some() {
                // pwsh is available
                assert_eq!(bin, "pwsh");
                assert_eq!(tool, "PowerShell");
            } else {
                // pwsh is not available - falls back to bash
                assert_eq!(bin, "bash");
                assert_eq!(tool, "Bash");
            }
        }
        // If PATH is not set, there's nothing to assert
    }

    /// Test resolve_shell_tool for bash
    #[test]
    fn test_resolve_shell_bash() {
        let (bin, arg, tool) = resolve_shell_tool(&FrontmatterShell::Bash);
        assert_eq!(bin, "bash");
        assert_eq!(arg, "-c");
        assert_eq!(tool, "Bash");
    }
}
