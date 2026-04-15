use super::Command;

pub fn create_extra_usage_command() -> Command {
    Command::local("extra-usage", "Manage extra usage").argument_hint("[buy|manage]")
}
