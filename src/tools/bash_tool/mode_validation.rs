// Source: ~/claudecode/openclaudecode/src/tools/BashTool/modeValidation.ts

/// Permission mode for tool execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolPermissionMode {
    /// Bypass all permission checks.
    BypassPermissions,
    /// Don't ask for permission (auto-allow or auto-deny based on rules).
    DontAsk,
    /// Auto-allow edits-related commands.
    AcceptEdits,
    /// Normal interactive mode -- ask for each command.
    Default,
}

/// Result of a permission check.
#[derive(Debug, Clone)]
pub enum PermissionBehavior {
    /// Allow the command without prompting.
    Allow {
        updated_input: serde_json::Value,
        decision_reason: DecisionReason,
    },
    /// Ask the user for permission.
    Ask,
    /// No mode-specific handling; pass through to normal permission flow.
    Passthrough {
        message: String,
    },
}

/// Reason for a permission decision.
#[derive(Debug, Clone)]
pub enum DecisionReason {
    Mode {
        mode: ToolPermissionMode,
    },
}

/// Context containing mode and permissions for tool execution.
#[derive(Debug, Clone)]
pub struct ToolPermissionContext {
    pub mode: ToolPermissionMode,
}

/// Input for the bash tool (simplified).
pub struct BashToolInput {
    pub command: String,
}

const ACCEPT_EDITS_ALLOWED_COMMANDS: &[&str] = &[
    "mkdir", "touch", "rm", "rmdir", "mv", "cp", "sed",
];

fn is_filesystem_command(command: &str) -> bool {
    ACCEPT_EDITS_ALLOWED_COMMANDS.contains(&command)
}

fn validate_command_for_mode(
    cmd: &str,
    context: &ToolPermissionContext,
) -> PermissionBehavior {
    let trimmed = cmd.trim();
    let base_cmd = trimmed.split_whitespace().next();

    let Some(base_cmd) = base_cmd else {
        return PermissionBehavior::Passthrough {
            message: "Base command not found".to_string(),
        };
    };

    // In Accept Edits mode, auto-allow filesystem operations
    if context.mode == ToolPermissionMode::AcceptEdits && is_filesystem_command(base_cmd) {
        return PermissionBehavior::Allow {
            updated_input: serde_json::json!({ "command": cmd }),
            decision_reason: DecisionReason::Mode {
                mode: ToolPermissionMode::AcceptEdits,
            },
        };
    }

    PermissionBehavior::Passthrough {
        message: format!("No mode-specific handling for '{base_cmd}' in {:?} mode", context.mode),
    }
}

/// Splits a command string into individual commands (simplified version).
fn split_commands(command: &str) -> Vec<&str> {
    // Simplified: split on `;`, `&&`, `||`, newlines
    command
        .split(|c: char| c == ';' || c == '\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Checks if commands should be handled differently based on the current permission mode.
///
/// This is the main entry point for mode-based permission logic.
/// Currently handles Accept Edits mode for filesystem commands,
/// but designed to be extended for other modes.
///
/// Returns:
/// - `PermissionBehavior::Allow` if the current mode permits auto-approval
/// - `PermissionBehavior::Ask` if the command needs approval in current mode
/// - `PermissionBehavior::Passthrough` if no mode-specific handling applies
pub fn check_permission_mode(
    input: &BashToolInput,
    context: &ToolPermissionContext,
) -> PermissionBehavior {
    // Skip if in bypass mode (handled elsewhere)
    if context.mode == ToolPermissionMode::BypassPermissions {
        return PermissionBehavior::Passthrough {
            message: "Bypass mode is handled in main permission flow".to_string(),
        };
    }

    // Skip if in dontAsk mode (handled in main permission flow)
    if context.mode == ToolPermissionMode::DontAsk {
        return PermissionBehavior::Passthrough {
            message: "DontAsk mode is handled in main permission flow".to_string(),
        };
    }

    let commands = split_commands(&input.command);

    // Check each subcommand
    for cmd in &commands {
        let result = validate_command_for_mode(cmd, context);

        // If any command triggers mode-specific behavior, return that result
        if !matches!(&result, PermissionBehavior::Passthrough { .. }) {
            return result;
        }
    }

    // No mode-specific handling needed
    PermissionBehavior::Passthrough {
        message: "No mode-specific validation required".to_string(),
    }
}

/// Get the list of commands that are auto-allowed in a given mode.
pub fn get_auto_allowed_commands(mode: &ToolPermissionMode) -> Vec<&'static str> {
    match mode {
        ToolPermissionMode::AcceptEdits => ACCEPT_EDITS_ALLOWED_COMMANDS.to_vec(),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bypass_mode_returns_passthrough() {
        let ctx = ToolPermissionContext {
            mode: ToolPermissionMode::BypassPermissions,
        };
        let input = BashToolInput {
            command: "mkdir test".to_string(),
        };
        assert!(matches!(
            check_permission_mode(&input, &ctx),
            PermissionBehavior::Passthrough { .. }
        ));
    }

    #[test]
    fn test_accept_edits_allows_mkdir() {
        let ctx = ToolPermissionContext {
            mode: ToolPermissionMode::AcceptEdits,
        };
        let input = BashToolInput {
            command: "mkdir test".to_string(),
        };
        assert!(matches!(
            check_permission_mode(&input, &ctx),
            PermissionBehavior::Allow { .. }
        ));
    }

    #[test]
    fn test_default_mode_passthrough() {
        let ctx = ToolPermissionContext {
            mode: ToolPermissionMode::Default,
        };
        let input = BashToolInput {
            command: "cargo test".to_string(),
        };
        assert!(matches!(
            check_permission_mode(&input, &ctx),
            PermissionBehavior::Passthrough { .. }
        ));
    }

    #[test]
    fn test_get_auto_allowed_commands() {
        assert_eq!(
            get_auto_allowed_commands(&ToolPermissionMode::AcceptEdits),
            vec!["mkdir", "touch", "rm", "rmdir", "mv", "cp", "sed"]
        );
        assert!(get_auto_allowed_commands(&ToolPermissionMode::Default).is_empty());
    }
}
