use super::Command;

pub fn create_pr_comments_command() -> Command {
    Command::local("pr-comments", "Manage PR comments")
}
