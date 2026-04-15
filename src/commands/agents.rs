// Source: /data/home/swei/claudecode/openclaudecode/src/cli/handlers/agents.ts
use super::Command;

pub fn create_agents_command() -> Command {
    Command::local("agents", "Manage agents").argument_hint("[list|create|delete] [<agent-id>]")
}
