use super::Command;

pub fn create_remote_setup_command() -> Command {
    Command::local("remote-setup", "Setup remote access")
}
