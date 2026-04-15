use super::Command;

pub fn create_issue_command() -> Command {
    Command::local("issue", "Manage issues").argument_hint("[create|list|close] [<issue-id>]")
}
