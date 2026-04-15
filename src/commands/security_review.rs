use super::Command;

pub fn create_security_review_command() -> Command {
    Command::prompt(
        "security-review",
        "Complete a security review of the pending changes on the current branch",
    )
}

const SECURITY_REVIEW_ALLOWED_TOOLS: &[&str] = &[
    "Bash(git diff:*)",
    "Bash(git status:*)",
    "Bash(git log:*)",
    "Bash(git show:*)",
    "Bash(git remote show:*)",
    "Read",
    "Glob",
    "Grep",
    "LS",
    "Task",
];

pub fn get_security_review_allowed_tools() -> Vec<String> {
    SECURITY_REVIEW_ALLOWED_TOOLS
        .iter()
        .map(|s| s.to_string())
        .collect()
}
