pub fn create_rewind_command() -> super::Command {
    super::Command::prompt("rewind", "Rewind conversation to a previous state")
}
