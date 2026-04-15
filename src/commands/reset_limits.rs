use super::Command;

pub fn create_reset_limits_command() -> Command {
    Command::local("reset-limits", "Reset usage limits")
}
