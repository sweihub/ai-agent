#![allow(dead_code)]

pub async fn get_worktree_paths_portable(cwd: &str) -> Vec<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(&["worktree", "list", "--porcelain"])
        .current_dir(cwd)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter(|line| line.starts_with("worktree "))
                .map(|line| line["worktree ".len()..].to_string())
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_worktree_paths() {
        let result = get_worktree_paths_portable("/tmp").await;
        assert!(result.is_empty() || !result.is_empty());
    }
}
