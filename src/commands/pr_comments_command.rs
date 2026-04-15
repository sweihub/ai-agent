pub fn create_pr_comments_command() -> super::Command {
    super::Command::prompt("pr-comments", "View PR comments")
}
