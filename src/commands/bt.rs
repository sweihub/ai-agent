use super::Command;

pub fn create_bt_command() -> Command {
    Command::local("bt", "Show backtrace")
}
