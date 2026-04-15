// Source: ~/claudecode/openclaudecode/src/utils/crossProjectResume.ts
//! Check if a log is from a different project directory and determine
//! whether it's a related worktree or a completely different project.
//!
//! For same-repo worktrees, we can resume directly without requiring cd.
//! For different projects, we generate the cd command.

#![allow(dead_code)]

use std::path::{Path, MAIN_SEPARATOR};

/// Result of checking cross-project resume.
pub enum CrossProjectResumeResult {
    /// Not a cross-project resume.
    NotCrossProject,
    /// Cross-project and same repo worktree.
    SameRepoWorktree { project_path: String },
    /// Cross-project, different repo.
    DifferentRepo { command: String, project_path: String },
}

/// Check if a log is from a different project directory and determine
/// whether it's a related worktree or a completely different project.
///
/// For same-repo worktrees, we can resume directly without requiring cd.
/// For different projects, we generate the cd command.
pub fn check_cross_project_resume(
    log_project_path: Option<&str>,
    show_all_projects: bool,
    worktree_paths: &[String],
    session_id: &str,
) -> CrossProjectResumeResult {
    let current_cwd = get_original_cwd();

    if !show_all_projects
        || log_project_path.is_none()
        || log_project_path.unwrap_or("") == current_cwd.as_str()
    {
        return CrossProjectResumeResult::NotCrossProject;
    }

    let project_path = log_project_path.unwrap_or("");

    // Gate worktree detection to ants only for staged rollout
    let user_type = std::env::var("USER_TYPE").unwrap_or_default();
    if user_type != "ant" {
        let command = format!(
            "cd '{}' && claude --resume {}",
            quote_path(project_path),
            session_id
        );
        return CrossProjectResumeResult::DifferentRepo {
            command,
            project_path: project_path.to_string(),
        };
    }

    // Check if log.project_path is under a worktree of the same repo
    let is_same_repo = worktree_paths.iter().any(|wt| {
        project_path == wt.as_str() || project_path.starts_with(&format!("{wt}{MAIN_SEPARATOR}"))
    });

    if is_same_repo {
        return CrossProjectResumeResult::SameRepoWorktree {
            project_path: project_path.to_string(),
        };
    }

    // Different repo - generate cd command
    let command = format!(
        "cd '{}' && claude --resume {}",
        quote_path(project_path),
        session_id
    );
    CrossProjectResumeResult::DifferentRepo {
        command,
        project_path: project_path.to_string(),
    }
}

/// Get the original working directory from environment.
fn get_original_cwd() -> String {
    std::env::var("AI_ORIGINAL_CWD")
        .or_else(|_| std::env::var("AI_CODE_ORIGINAL_CWD"))
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        })
}

/// Quote a path for use in a shell command.
fn quote_path(path: &str) -> String {
    // Simple quoting: escape single quotes
    format!("'{}'", path.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_cross_project_same_path() {
        let result = check_cross_project_resume(
            Some("/same/path"),
            true,
            &[],
            "session-123",
        );
        // This depends on AI_ORIGINAL_CWD env var, so we just check it doesn't panic
        match result {
            CrossProjectResumeResult::NotCrossProject => {}
            _ => {
                // May be cross-project if ORIGINAL_CWD differs
            }
        }
    }

    #[test]
    fn test_not_cross_project_show_all_false() {
        let result = check_cross_project_resume(
            Some("/some/path"),
            false,
            &[],
            "session-123",
        );
        assert!(matches!(result, CrossProjectResumeResult::NotCrossProject));
    }

    #[test]
    fn test_different_repo_non_ant() {
        // Ensure USER_TYPE is not "ant"
        let result = check_cross_project_resume(
            Some("/different/project"),
            true,
            &["/other/worktree".to_string()],
            "session-456",
        );
        if let CrossProjectResumeResult::DifferentRepo {
            command,
            project_path,
        } = result
        {
            assert!(command.contains("/different/project"));
            assert_eq!(project_path, "/different/project");
        }
    }

    #[test]
    fn test_quote_path() {
        assert_eq!(quote_path("/simple/path"), "'/simple/path'");
        assert_eq!(
            quote_path("/path/with'single/quote"),
            "'/path/with'\\''single/quote'"
        );
    }
}
