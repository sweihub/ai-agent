use std::process::Command;

pub fn get_worktree_paths_portable(cwd: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(cwd)
        .output();

    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.starts_with("worktree "))
            .map(|line| line["worktree ".len()..].to_string())
            .collect(),
        _ => Vec::new(),
    }
}
