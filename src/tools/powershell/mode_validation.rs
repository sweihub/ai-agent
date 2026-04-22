// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/modeValidation.ts
//! PowerShell permission mode validation.
//!
//! Checks if commands should be auto-allowed based on the current permission mode.
//! In acceptEdits mode, filesystem-modifying PowerShell cmdlets are auto-allowed.

use once_cell::sync::Lazy;
use std::collections::HashSet;

use super::read_only_validation::{
    is_cwd_changing_cmdlet, is_safe_output_command, resolve_to_canonical,
};

/// Filesystem-modifying cmdlets that are auto-allowed in acceptEdits mode
static ACCEPT_EDITS_ALLOWED_CMDLETS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("set-content");
    set.insert("add-content");
    set.insert("remove-item");
    set.insert("clear-content");
    set
});

/// Link-creating -ItemType values
static LINK_ITEM_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut set = HashSet::new();
    set.insert("symboliclink");
    set.insert("junction");
    set.insert("hardlink");
    set
});

/// Check if cmdlet is allowed in acceptEdits mode
fn is_accept_edits_allowed_cmdlet(name: &str) -> bool {
    let canonical = resolve_to_canonical(name);
    ACCEPT_EDITS_ALLOWED_CMDLETS.contains(canonical.as_str())
}

/// Check if a lowered, dash-normalized arg is an unambiguous PowerShell
/// abbreviation of New-Item's -ItemType or -Type param.
fn is_item_type_param_abbrev(param: &str) -> bool {
    let lower = param.to_lowercase();
    (lower.len() >= 3 && lower.starts_with("-it"))
        || (lower.len() >= 3
            && (lower == "-ty" || lower.starts_with("-typ") || lower.starts_with("-type")))
}

/// Detects New-Item creating a filesystem link
pub fn is_symlink_creating_command(name: &str, args: &[String]) -> bool {
    let canonical = resolve_to_canonical(name);
    if canonical != "new-item" {
        return false;
    }

    let mut i = 0;
    while i < args.len() {
        let raw = &args[i];
        if raw.is_empty() {
            i += 1;
            continue;
        }

        // Normalize dash prefixes
        let normalized = if raw.starts_with('-')
            || raw.starts_with('–')
            || raw.starts_with('—')
            || raw.starts_with('―')
            || raw.starts_with('/')
        {
            format!("-{}", &raw[1..])
        } else {
            raw.clone()
        };

        let lower = normalized.to_lowercase();

        // Split colon-bound value: -it:SymbolicLink
        let colon_idx = lower[1..].find(':').map(|p| p + 1).unwrap_or(0);
        let param_raw = if colon_idx > 0 {
            lower.get(1..=colon_idx).unwrap_or(&lower).to_string()
        } else {
            lower.clone()
        };

        // Strip backtick escapes
        let param = param_raw.replace('`', "");

        if !is_item_type_param_abbrev(&param) {
            i += 1;
            continue;
        }

        // Get value
        let raw_val = if colon_idx > 0 {
            lower.get(colon_idx + 1..).unwrap_or("").to_string()
        } else {
            args.get(i + 1)
                .map(|s| s.to_lowercase())
                .unwrap_or_default()
        };

        // Strip backtick and quotes
        let val = raw_val
            .replace('`', "")
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();

        if LINK_ITEM_TYPES.contains(val.as_str()) {
            return true;
        }

        i += 1;
    }

    false
}

/// Permission result behavior
#[derive(Debug, Clone)]
pub enum PermissionBehavior {
    Allow,
    Deny,
    Ask,
    Passthrough,
}

/// Permission result
#[derive(Debug, Clone)]
pub struct PermissionModeResult {
    pub behavior: PermissionBehavior,
    pub message: String,
}

impl PermissionModeResult {
    pub fn allow() -> Self {
        Self {
            behavior: PermissionBehavior::Allow,
            message: "Auto-allowed in acceptEdits mode".to_string(),
        }
    }

    pub fn deny(message: &str) -> Self {
        Self {
            behavior: PermissionBehavior::Deny,
            message: message.to_string(),
        }
    }

    pub fn ask(message: &str) -> Self {
        Self {
            behavior: PermissionBehavior::Ask,
            message: message.to_string(),
        }
    }

    pub fn passthrough(message: &str) -> Self {
        Self {
            behavior: PermissionBehavior::Passthrough,
            message: message.to_string(),
        }
    }
}

/// Checks if commands should be handled differently based on the current permission mode
pub fn check_permission_mode(command: &str, mode: &str) -> PermissionModeResult {
    // Skip bypass and dontAsk modes
    if mode == "bypassPermissions" || mode == "dontAsk" {
        return PermissionModeResult::passthrough("Mode is handled in main permission flow");
    }

    if mode != "acceptEdits" {
        return PermissionModeResult::passthrough("No mode-specific validation required");
    }

    // Check for security concerns that require approval
    use super::read_only_validation::has_sync_security_concerns;
    if has_sync_security_concerns(command) {
        return PermissionModeResult::passthrough(
            "Command contains subexpressions, script blocks, or member invocations that require approval",
        );
    }

    // Check for compound command with cwd change
    let parts: Vec<&str> = command.split(|c| c == ';' || c == '|').collect();
    if parts.len() > 1 {
        let mut has_cd = false;
        let mut has_write = false;
        let mut has_symlink = false;

        for part in &parts {
            let first_word = part.trim().split_whitespace().next().unwrap_or("");
            if is_cwd_changing_cmdlet(first_word) {
                has_cd = true;
            }
            if is_accept_edits_allowed_cmdlet(first_word) {
                has_write = true;
            }
            // Check for symlink creation
            let args: Vec<String> = part
                .trim()
                .split_whitespace()
                .skip(1)
                .map(String::from)
                .collect();
            if is_symlink_creating_command(first_word, &args) {
                has_symlink = true;
            }
        }

        if has_cd && has_write {
            return PermissionModeResult::passthrough(
                "Compound command contains a directory-changing command with a write operation",
            );
        }

        if has_symlink {
            return PermissionModeResult::passthrough("Compound command creates a filesystem link");
        }
    }

    // Check if the command is an acceptEdits-allowed cmdlet
    let first_word = command.trim().split_whitespace().next().unwrap_or("");
    if is_accept_edits_allowed_cmdlet(first_word) {
        // Additional checks for safe arguments
        use super::read_only_validation::arg_leaks_value;

        let args: Vec<&str> = command.trim().split_whitespace().skip(1).collect();
        for arg in args {
            // Skip flags
            if arg.starts_with('-') {
                continue;
            }
            if arg_leaks_value(arg) {
                return PermissionModeResult::passthrough(
                    "Command contains potentially unsafe arguments",
                );
            }
        }

        return PermissionModeResult::allow();
    }

    PermissionModeResult::passthrough("Command not in acceptEdits allowlist")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_accept_edits_allowed_cmdlet() {
        assert!(is_accept_edits_allowed_cmdlet("set-content"));
        assert!(is_accept_edits_allowed_cmdlet("remove-item"));
        assert!(!is_accept_edits_allowed_cmdlet("get-content"));
    }

    #[test]
    fn test_is_symlink_creating_command() {
        assert!(is_symlink_creating_command(
            "new-item",
            &["-ItemType".to_string(), "SymbolicLink".to_string()]
        ));
        assert!(is_symlink_creating_command(
            "ni",
            &["-ItemType".to_string(), "Junction".to_string()]
        ));
        assert!(!is_symlink_creating_command(
            "new-item",
            &["-ItemType".to_string(), "File".to_string()]
        ));
        assert!(!is_symlink_creating_command("get-content", &[]));
    }

    #[test]
    fn test_check_permission_mode() {
        let result = check_permission_mode("Get-Content test.txt", "readOnly");
        assert!(matches!(result.behavior, PermissionBehavior::Passthrough));

        let result = check_permission_mode("Set-Content test.txt 'hello'", "acceptEdits");
        assert!(matches!(result.behavior, PermissionBehavior::Allow));

        let result = check_permission_mode("$(malicious)", "acceptEdits");
        assert!(matches!(result.behavior, PermissionBehavior::Passthrough));
    }
}
