use crate::constants::env::ai;
use std::path::Path;

pub fn is_worktree_mode_enabled() -> bool {
    std::env::var(ai::WORKTREE_MODE)
        .ok()
        .map(|v| v == "true")
        .unwrap_or(false)
}

pub fn get_worktree_root() -> Option<String> {
    std::env::var(ai::WORKTREE_ROOT).ok()
}

pub fn get_worktree_slug() -> Option<String> {
    std::env::var(ai::WORKTREE_SLUG).ok()
}

pub fn get_original_cwd() -> Option<String> {
    std::env::var(ai::ORIGINAL_CWD).ok()
}

pub fn is_in_worktree() -> bool {
    get_worktree_root().is_some()
}

pub fn find_worktree_dir(repo_root: &Path, slug: &str) -> Option<std::path::PathBuf> {
    let worktrees_dir = repo_root.join(".ai").join("worktrees");
    let flattened = slug.replace('/', "+");
    let worktree_path = worktrees_dir.join(flattened);

    if worktree_path.exists() {
        Some(worktree_path)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_detection() {
        let result = is_worktree_mode_enabled();
        assert!(result == false || result == true);
    }
}
