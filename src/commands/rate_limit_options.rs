use super::Command;

pub fn create_rate_limit_options_command() -> Command {
    Command::local("rate-limit-options", "Configure rate limiting")
}
