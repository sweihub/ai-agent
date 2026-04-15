use super::Command;

pub fn create_agents_platform_command() -> Command {
    Command::local("agents-platform", "Manage agents platform")
        .argument_hint("[list|create|delete] [<agent-id>]")
}
