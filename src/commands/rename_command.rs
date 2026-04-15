pub fn create_rename_command() -> super::Command {
    super::Command::prompt("rename", "Rename the current session")
}
