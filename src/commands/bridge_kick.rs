use super::Command;

pub fn create_bridge_kick_command() -> Command {
    Command::local("bridge-kick", "Kick from bridged session").argument_hint("<session-id>")
}
