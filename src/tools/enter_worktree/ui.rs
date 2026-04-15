// Source: ~/claudecode/openclaudecode/src/tools/EnterWorktreeTool/UI.tsx
/// Rendered tool-use message text for EnterWorktree.
pub fn render_tool_use_message() -> &'static str {
    "Creating worktree..."
}

/// Rendered tool-result message for EnterWorktree.
/// Returns a formatted string showing the worktree branch and path.
pub fn render_tool_result_message(worktree_branch: &str, worktree_path: &str) -> String {
    format!(
        "Switched to worktree on branch {worktree_branch}\n{worktree_path}",
        worktree_branch = worktree_branch,
        worktree_path = worktree_path,
    )
}

/// Output type for the EnterWorktree tool.
#[derive(Debug, Clone)]
pub struct EnterWorktreeOutput {
    pub worktree_branch: String,
    pub worktree_path: String,
}
