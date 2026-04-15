use super::Command;

pub fn create_good_claude_command() -> Command {
    Command::prompt("good-claude", "Find good Claude behavior patterns")
}
