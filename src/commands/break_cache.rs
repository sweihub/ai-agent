use super::Command;

pub fn create_break_cache_command() -> Command {
    Command::local("break-cache", "Break context cache")
}
