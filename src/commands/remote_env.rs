use super::Command;

pub fn create_remote_env_command() -> Command {
    Command::local("remote-env", "Manage remote environment")
}
