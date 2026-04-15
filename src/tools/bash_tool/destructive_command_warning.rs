// Source: ~/claudecode/openclaudecode/src/tools/BashTool/destructiveCommandWarning.ts
use regex::Regex;

/// A destructive pattern with its associated warning message.
struct DestructivePattern {
    pattern: fn() -> Regex,
    warning: &'static str,
}

lazy_static::lazy_static! {
    static ref GIT_RESET_HARD: Regex = Regex::new(r"\bgit\s+reset\s+--hard\b").unwrap();
    static ref GIT_PUSH_FORCE: Regex = Regex::new(r"\bgit\s+push\b[^;&|\n]*[ \t](--force|--force-with-lease|-f)\b").unwrap();
    static ref GIT_CLEAN_FORCE: Regex = Regex::new(r"\bgit\s+clean\b(?![^;&|\n]*(?:-[a-zA-Z]*n|--dry-run))[^;&|\n]*-[a-zA-Z]*f").unwrap();
    static ref GIT_CHECKOUT_DOT: Regex = Regex::new(r"\bgit\s+checkout\s+(--\s+)?\.[ \t]*($|[;&|\n])").unwrap();
    static ref GIT_RESTORE_DOT: Regex = Regex::new(r"\bgit\s+restore\s+(--\s+)?\.[ \t]*($|[;&|\n])").unwrap();
    static ref GIT_STASH_DROP_CLEAR: Regex = Regex::new(r"\bgit\s+stash[ \t]+(drop|clear)\b").unwrap();
    static ref GIT_BRANCH_FORCE_DELETE: Regex = Regex::new(r"\bgit\s+branch\s+(-D[ \t]|--delete\s+--force|--force\s+--delete)\b").unwrap();
    static ref GIT_NO_VERIFY: Regex = Regex::new(r"\bgit\s+(commit|push|merge)\b[^;&|\n]*--no-verify\b").unwrap();
    static ref GIT_AMEND: Regex = Regex::new(r"\bgit\s+commit\b[^;&|\n]*--amend\b").unwrap();
    static ref RM_RF: Regex = Regex::new(r"(^|[;&|\n]\s*)rm\s+-[a-zA-Z]*[rR][a-zA-Z]*f|(^|[;&|\n]\s*)rm\s+-[a-zA-Z]*f[a-zA-Z]*[rR]").unwrap();
    static ref RM_R: Regex = Regex::new(r"(^|[;&|\n]\s*)rm\s+-[a-zA-Z]*[rR]").unwrap();
    static ref RM_F: Regex = Regex::new(r"(^|[;&|\n]\s*)rm\s+-[a-zA-Z]*f").unwrap();
    static ref DROP_TRUNCATE: Regex = Regex::new(r"\b(DROP|TRUNCATE)\s+(TABLE|DATABASE|SCHEMA)\b").unwrap();
    static ref DELETE_FROM: Regex = Regex::new(r"\bDELETE\s+FROM\s+\w+[ \t]*(;|\"|'|\n|$)").unwrap();
    static ref KUBECTL_DELETE: Regex = Regex::new(r"\bkubectl\s+delete\b").unwrap();
    static ref TERRAFORM_DESTROY: Regex = Regex::new(r"\bterraform\s+destroy\b").unwrap();
}

/// Destructive patterns defined as lazy statics with their warnings.
const PATTERNS: &[(fn() -> &'static Regex, &'static str)] = &[
    // Git -- data loss / hard to reverse
    (|| &GIT_RESET_HARD, "Note: may discard uncommitted changes"),
    (|| &GIT_PUSH_FORCE, "Note: may overwrite remote history"),
    (|| &GIT_CLEAN_FORCE, "Note: may permanently delete untracked files"),
    (|| &GIT_CHECKOUT_DOT, "Note: may discard all working tree changes"),
    (|| &GIT_RESTORE_DOT, "Note: may discard all working tree changes"),
    (|| &GIT_STASH_DROP_CLEAR, "Note: may permanently remove stashed changes"),
    (|| &GIT_BRANCH_FORCE_DELETE, "Note: may force-delete a branch"),

    // Git -- safety bypass
    (|| &GIT_NO_VERIFY, "Note: may skip safety hooks"),
    (|| &GIT_AMEND, "Note: may rewrite the last commit"),

    // File deletion (dangerous paths already handled by checkDangerousRemovalPaths)
    (|| &RM_RF, "Note: may recursively force-remove files"),
    (|| &RM_R, "Note: may recursively remove files"),
    (|| &RM_F, "Note: may force-remove files"),

    // Database
    (|| &DROP_TRUNCATE, "Note: may drop or truncate database objects"),
    (|| &DELETE_FROM, "Note: may delete all rows from a database table"),

    // Infrastructure
    (|| &KUBECTL_DELETE, "Note: may delete Kubernetes resources"),
    (|| &TERRAFORM_DESTROY, "Note: may destroy Terraform infrastructure"),
];

/// Checks if a bash command matches known destructive patterns.
/// Returns a human-readable warning string, or `None` if no destructive pattern is detected.
pub fn get_destructive_command_warning(command: &str) -> Option<&'static str> {
    for (get_regex, warning) in PATTERNS {
        if get_regex().is_match(command) {
            return Some(warning);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_reset_hard() {
        assert_eq!(
            get_destructive_command_warning("git reset --hard HEAD"),
            Some("Note: may discard uncommitted changes")
        );
    }

    #[test]
    fn test_git_push_force() {
        assert_eq!(
            get_destructive_command_warning("git push --force"),
            Some("Note: may overwrite remote history")
        );
    }

    #[test]
    fn test_safe_command() {
        assert_eq!(get_destructive_command_warning("git status"), None);
    }

    #[test]
    fn test_rm_rf() {
        assert_eq!(
            get_destructive_command_warning("rm -rf /tmp/foo"),
            Some("Note: may recursively force-remove files")
        );
    }

    #[test]
    fn test_drop_table() {
        assert_eq!(
            get_destructive_command_warning("DROP TABLE users"),
            Some("Note: may drop or truncate database objects")
        );
    }

    #[test]
    fn test_terraform_destroy() {
        assert_eq!(
            get_destructive_command_warning("terraform destroy"),
            Some("Note: may destroy Terraform infrastructure")
        );
    }
}
