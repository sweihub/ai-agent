use super::Command;

pub fn create_reload_plugins_command() -> Command {
    Command::local(
        "reload-plugins",
        "Activate pending plugin changes in the current session",
    )
}
