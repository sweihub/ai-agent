pub fn create_branch_command() -> super::Command {
    super::Command::prompt("branch", "Show current git branch")
}
