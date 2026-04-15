use super::Command;

pub fn create_release_notes_command() -> Command {
    Command::local("release-notes", "View release notes").supports_non_interactive(true)
}
