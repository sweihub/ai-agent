// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/destructiveCommandWarning.ts
//! Detects potentially destructive PowerShell commands and returns a warning

use once_cell::sync::Lazy;
use regex::Regex;

/// Destructive pattern with warning message
struct DestructivePattern {
    pattern: Regex,
    warning: &'static str,
}

/// All destructive patterns
fn get_destructive_patterns() -> Vec<DestructivePattern> {
    vec![
        // Remove-Item with -Recurse and/or -Force
        DestructivePattern {
            pattern: Regex::new(r"(?:^|[|;&\n({])\s*(Remove-Item|rm|del|rd|rmdir|ri)\b[^|;&\n}]*-Recurse\b[^|;&\n}]*-Force\b").unwrap(),
            warning: "Note: may recursively force-remove files",
        },
        DestructivePattern {
            pattern: Regex::new(r"(?:^|[|;&\n({])\s*(Remove-Item|rm|del|rd|rmdir|ri)\b[^|;&\n}]*-Force\b[^|;&\n}]*-Recurse\b").unwrap(),
            warning: "Note: may recursively force-remove files",
        },
        DestructivePattern {
            pattern: Regex::new(r"(?:^|[|;&\n({])\s*(Remove-Item|rm|del|rd|rmdir|ri)\b[^|;&\n}]*-Recurse\b").unwrap(),
            warning: "Note: may recursively remove files",
        },
        DestructivePattern {
            pattern: Regex::new(r"(?:^|[|;&\n({])\s*(Remove-Item|rm|del|rd|rmdir|ri)\b[^|;&\n}]*-Force\b").unwrap(),
            warning: "Note: may force-remove files",
        },
        // Clear-Content on broad paths
        DestructivePattern {
            pattern: Regex::new(r"\bClear-Content\b[^|;&\n]*\*").unwrap(),
            warning: "Note: may clear content of multiple files",
        },
        // Format-Volume and Clear-Disk
        DestructivePattern {
            pattern: Regex::new(r"\bFormat-Volume\b").unwrap(),
            warning: "Note: may format a disk volume",
        },
        DestructivePattern {
            pattern: Regex::new(r"\bClear-Disk\b").unwrap(),
            warning: "Note: may clear a disk",
        },
        // Git destructive operations
        DestructivePattern {
            pattern: Regex::new(r"\bgit\s+reset\s+--hard\b").unwrap(),
            warning: "Note: may discard uncommitted changes",
        },
        DestructivePattern {
            pattern: Regex::new(r"\bgit\s+push\b[^|;&\n]*\s+(--force|--force-with-lease|-f)\b").unwrap(),
            warning: "Note: may overwrite remote history",
        },
        // Note: git clean -f pattern handled manually due to negative lookahead
        DestructivePattern {
            pattern: Regex::new(r"\bgit\s+stash\s+(drop|clear)\b").unwrap(),
            warning: "Note: may permanently remove stashed changes",
        },
        // Database operations
        DestructivePattern {
            pattern: Regex::new(r"\b(DROP|TRUNCATE)\s+(TABLE|DATABASE|SCHEMA)\b").unwrap(),
            warning: "Note: may drop or truncate database objects",
        },
        // System operations
        DestructivePattern {
            pattern: Regex::new(r"\bStop-Computer\b").unwrap(),
            warning: "Note: will shut down the computer",
        },
        DestructivePattern {
            pattern: Regex::new(r"\bRestart-Computer\b").unwrap(),
            warning: "Note: will restart the computer",
        },
        DestructivePattern {
            pattern: Regex::new(r"\bClear-RecycleBin\b").unwrap(),
            warning: "Note: permanently deletes recycled files",
        },
    ]
}

static DESTRUCTIVE_PATTERNS: Lazy<Vec<DestructivePattern>> = Lazy::new(get_destructive_patterns);

/// Checks if a PowerShell command matches known destructive patterns.
/// Returns a human-readable warning string, or None if no destructive pattern is detected.
pub fn get_destructive_command_warning(command: &str) -> Option<&'static str> {
    for pattern in DESTRUCTIVE_PATTERNS.iter() {
        if pattern.pattern.is_match(command) {
            return Some(pattern.warning);
        }
    }

    // Manual check for git clean -f (without -n or --dry-run)
    let lower = command.to_lowercase();
    if lower.contains("git") && lower.contains("clean") {
        // Check for -f without -n or --dry-run
        let has_f = lower.contains(" -f ")
            || lower.contains(" -f\n")
            || lower.contains(" -f\t")
            || lower.contains(" --force")
            || lower.contains(" -fd");
        let has_dry_run = lower.contains(" -n ") || lower.contains(" --dry-run");
        if has_f && !has_dry_run {
            return Some("Note: may permanently delete untracked files");
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_item_recursive_force() {
        let warning = get_destructive_command_warning("Remove-Item -Path ./foo -Recurse -Force");
        assert!(warning.is_some());
    }

    #[test]
    fn test_git_reset_hard() {
        let warning = get_destructive_command_warning("git reset --hard");
        assert!(warning.is_some());
    }

    #[test]
    fn test_safe_command() {
        let warning = get_destructive_command_warning("Get-ChildItem");
        assert!(warning.is_none());
    }
}
