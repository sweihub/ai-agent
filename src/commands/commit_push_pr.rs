use super::Command;

const ALLOWED_TOOLS: &[&str] = &[
    "Bash(git checkout --branch:*)",
    "Bash(git checkout -b:*)",
    "Bash(git add:*)",
    "Bash(git status:*)",
    "Bash(git push:*)",
    "Bash(git commit:*)",
    "Bash(gh pr create:*)",
    "Bash(gh pr edit:*)",
    "Bash(gh pr view:*)",
    "Bash(gh pr merge:*)",
    "ToolSearch",
    "mcp__slack__send_message",
    "mcp__claude_ai_Slack__slack_send_message",
];

pub fn create_commit_push_pr_command() -> Command {
    Command::prompt("commit-push-pr", "Commit, push, and open a PR")
        .argument_hint("[<additional-instructions>]")
}

pub fn get_commit_push_pr_allowed_tools() -> Vec<String> {
    ALLOWED_TOOLS.iter().map(|s| s.to_string()).collect()
}
