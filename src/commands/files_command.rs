pub fn create_files_command() -> super::Command {
    super::Command::prompt("files", "List files in the project")
}
