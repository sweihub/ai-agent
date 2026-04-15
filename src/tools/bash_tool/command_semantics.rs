// Source: ~/claudecode/openclaudecode/src/tools/BashTool/commandSemantics.ts
use std::collections::HashMap;

/// Command semantics configuration for interpreting exit codes in different contexts.
///
/// Many commands use exit codes to convey information other than just success/failure.
/// For example, grep returns 1 when no matches are found, which is not an error condition.

/// Result of interpreting a command's exit code.
#[derive(Debug, Clone)]
pub struct CommandSemanticResult {
    pub is_error: bool,
    pub message: Option<String>,
}

/// Type alias for command semantic functions.
type CommandSemantic = fn(exit_code: i32, _stdout: &str, _stderr: &str) -> CommandSemanticResult;

/// Default semantic: treat only 0 as success, everything else as error.
fn default_semantic(exit_code: i32, _stdout: &str, _stderr: &str) -> CommandSemanticResult {
    CommandSemanticResult {
        is_error: exit_code != 0,
        message: if exit_code != 0 {
            Some(format!("Command failed with exit code {exit_code}"))
        } else {
            None
        },
    }
}

/// Get the semantic interpretation for a command.
fn get_command_semantic(command: &str) -> CommandSemantic {
    let base_command = heuristically_extract_base_command(command);
    COMMAND_SEMANTICS
        .get(&base_command)
        .copied()
        .unwrap_or(default_semantic)
}

/// Extract just the command name (first word) from a single command string.
fn extract_base_command(command: &str) -> String {
    command
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

/// Splits a command string into individual commands (simplified version).
fn split_commands(command: &str) -> Vec<&str> {
    command
        .split(|c: char| c == ';' || c == '\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Extract the primary command from a complex command line;
/// May get it super wrong - don't depend on this for security.
fn heuristically_extract_base_command(command: &str) -> String {
    let segments = split_commands(command);

    // Take the last command as that's what determines the exit code
    let last_command = segments.last().copied().unwrap_or(command);

    extract_base_command(last_command)
}

lazy_static::lazy_static! {
    static ref COMMAND_SEMANTICS: HashMap<&'static str, CommandSemantic> = {
        let mut m = HashMap::new();

        // grep: 0=matches found, 1=no matches, 2+=error
        m.insert("grep", |exit_code: i32, _stdout: &str, _stderr: &str| {
            CommandSemanticResult {
                is_error: exit_code >= 2,
                message: if exit_code == 1 { Some("No matches found".to_string()) } else { None },
            }
        });

        // ripgrep has same semantics as grep
        m.insert("rg", |exit_code: i32, _stdout: &str, _stderr: &str| {
            CommandSemanticResult {
                is_error: exit_code >= 2,
                message: if exit_code == 1 { Some("No matches found".to_string()) } else { None },
            }
        });

        // find: 0=success, 1=partial success (some dirs inaccessible), 2+=error
        m.insert("find", |exit_code: i32, _stdout: &str, _stderr: &str| {
            CommandSemanticResult {
                is_error: exit_code >= 2,
                message: if exit_code == 1 { Some("Some directories were inaccessible".to_string()) } else { None },
            }
        });

        // diff: 0=no differences, 1=differences found, 2+=error
        m.insert("diff", |exit_code: i32, _stdout: &str, _stderr: &str| {
            CommandSemanticResult {
                is_error: exit_code >= 2,
                message: if exit_code == 1 { Some("Files differ".to_string()) } else { None },
            }
        });

        // test/[: 0=condition true, 1=condition false, 2+=error
        m.insert("test", |exit_code: i32, _stdout: &str, _stderr: &str| {
            CommandSemanticResult {
                is_error: exit_code >= 2,
                message: if exit_code == 1 { Some("Condition is false".to_string()) } else { None },
            }
        });

        // [ is an alias for test
        m.insert("[", |exit_code: i32, _stdout: &str, _stderr: &str| {
            CommandSemanticResult {
                is_error: exit_code >= 2,
                message: if exit_code == 1 { Some("Condition is false".to_string()) } else { None },
            }
        });

        // wc, head, tail, cat, etc.: these typically only fail on real errors
        // so we use default semantics (not explicitly listed)

        m
    };
}

/// Interpret command result based on semantic rules.
pub fn interpret_command_result(
    command: &str,
    exit_code: i32,
    stdout: &str,
    stderr: &str,
) -> CommandSemanticResult {
    let semantic = get_command_semantic(command);
    semantic(exit_code, stdout, stderr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grep_no_matches() {
        let result = interpret_command_result("grep pattern file.txt", 1, "", "");
        assert!(!result.is_error);
        assert_eq!(result.message, Some("No matches found".to_string()));
    }

    #[test]
    fn test_grep_error() {
        let result = interpret_command_result("grep pattern file.txt", 2, "", "grep: file.txt: No such file");
        assert!(result.is_error);
    }

    #[test]
    fn test_grep_match() {
        let result = interpret_command_result("grep pattern file.txt", 0, "matched line", "");
        assert!(!result.is_error);
        assert_eq!(result.message, None);
    }

    #[test]
    fn test_diff_files_differ() {
        let result = interpret_command_result("diff a.txt b.txt", 1, "", "");
        assert!(!result.is_error);
        assert_eq!(result.message, Some("Files differ".to_string()));
    }

    #[test]
    fn test_unknown_command_default_semantic() {
        let result = interpret_command_result("unknown_cmd", 1, "", "");
        assert!(result.is_error);
    }

    #[test]
    fn test_unknown_command_success() {
        let result = interpret_command_result("unknown_cmd", 0, "output", "");
        assert!(!result.is_error);
        assert_eq!(result.message, None);
    }
}
