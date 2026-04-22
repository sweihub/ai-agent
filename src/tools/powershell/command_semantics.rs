// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/commandSemantics.ts
//! Command semantics configuration for interpreting exit codes in PowerShell.
//!
//! PowerShell-native cmdlets do NOT need exit-code semantics:
//!   - Select-String (grep equivalent) exits 0 on no-match (returns $null)
//!   - Compare-Object (diff equivalent) exits 0 regardless
//!   - Test-Path exits 0 regardless (returns bool via pipeline)
//! Native cmdlets signal failure via terminating errors ($?), not exit codes.
//!
//! However, EXTERNAL executables invoked from PowerShell DO set $LASTEXITCODE,
//! and many use non-zero codes to convey information rather than failure:
//!   - grep.exe / rg.exe (Git for Windows, scoop, etc.): 1 = no match
//!   - findstr.exe (Windows native): 1 = no match
//!   - robocopy.exe (Windows native): 0-7 = success, 8+ = error (notorious!)

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Result of command interpretation
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub is_error: bool,
    pub message: Option<String>,
}

/// Command type identifier for lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandType {
    Default,
    Grep,
    Robocopy,
}

/// Get command type from string
fn get_command_type(base_command: &str) -> CommandType {
    match base_command {
        "grep" | "rg" | "findstr" => CommandType::Grep,
        "robocopy" => CommandType::Robocopy,
        _ => CommandType::Default,
    }
}

/// Interpret command result based on semantic rules
pub fn interpret_command_result(
    command: &str,
    exit_code: i32,
    stdout: &str,
    stderr: &str,
) -> CommandResult {
    let base_command = extract_base_command(command);
    let cmd_type = get_command_type(&base_command);

    match cmd_type {
        CommandType::Grep => {
            if exit_code >= 2 {
                CommandResult {
                    is_error: true,
                    message: Some(format!("Command failed with exit code {}", exit_code)),
                }
            } else if exit_code == 1 {
                CommandResult {
                    is_error: false,
                    message: Some("No matches found".to_string()),
                }
            } else {
                CommandResult {
                    is_error: false,
                    message: None,
                }
            }
        }
        CommandType::Robocopy => {
            if exit_code >= 8 {
                CommandResult {
                    is_error: true,
                    message: Some(format!("Robocopy failed with exit code {}", exit_code)),
                }
            } else if exit_code == 0 {
                CommandResult {
                    is_error: false,
                    message: Some("No files copied (already in sync)".to_string()),
                }
            } else {
                // 1-7 are success codes
                let message = if exit_code & 1 != 0 {
                    "Files copied successfully".to_string()
                } else {
                    "Robocopy completed (no errors)".to_string()
                };
                CommandResult {
                    is_error: false,
                    message: Some(message),
                }
            }
        }
        CommandType::Default => {
            if exit_code == 0 {
                CommandResult {
                    is_error: false,
                    message: None,
                }
            } else {
                CommandResult {
                    is_error: true,
                    message: Some(format!("Command failed with exit code {}", exit_code)),
                }
            }
        }
    }
}

/// Extract the command name from a single pipeline segment.
/// Strips leading `&` / `.` call operators and `.exe` suffix, lowercases.
fn extract_base_command(command: &str) -> String {
    let segments: Vec<&str> = command
        .split(|c| c == ';' || c == '|')
        .filter(|s| !s.trim().is_empty())
        .collect();
    let last = segments.last().unwrap_or(&command);

    // Strip PowerShell call operators: & "cmd", . "cmd"
    let stripped = last.trim().trim_start_matches(&['&', '.'][..]).trim();
    let first_token = stripped.split_whitespace().next().unwrap_or("");
    // Strip surrounding quotes if command was invoked as & "grep.exe"
    let unquoted = first_token.trim_matches('"').trim_matches('\'');
    // Strip path: C:\bin\grep.exe → grep.exe, .\rg.exe → rg.exe
    let basename = unquoted
        .rsplit(|c| c == '\\' || c == '/')
        .next()
        .unwrap_or(unquoted);
    // Strip .exe suffix (Windows is case-insensitive)
    basename.to_lowercase().trim_end_matches(".exe").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_success() {
        let result = interpret_command_result("grep pattern file.txt", 0, "match\n", "");
        assert!(!result.is_error);
    }

    #[test]
    fn test_grep_no_match() {
        let result = interpret_command_result("grep pattern file.txt", 1, "", "");
        assert!(!result.is_error);
        assert_eq!(result.message, Some("No matches found".to_string()));
    }

    #[test]
    fn test_robocopy_success() {
        let result = interpret_command_result("robocopy src dst", 1, "", "");
        assert!(!result.is_error);
        assert_eq!(
            result.message,
            Some("Files copied successfully".to_string())
        );
    }

    #[test]
    fn test_robocopy_failure() {
        let result = interpret_command_result("robocopy src dst", 16, "", "");
        assert!(result.is_error);
    }
}
