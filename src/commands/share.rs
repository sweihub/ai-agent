use super::Command;

pub fn create_share_command() -> Command {
    Command::local("share", "Share session")
}
