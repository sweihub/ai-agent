use super::Command;

pub fn create_autofix_pr_command() -> Command {
    Command::local("autofix-pr", "Auto-fix PR").argument_hint("<pr-number>")
}
