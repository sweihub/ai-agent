use super::Command;

pub fn create_sandbox_toggle_command() -> Command {
    Command::local("sandbox", "Configure sandbox settings")
        .argument_hint("exclude \"command pattern\"")
}
