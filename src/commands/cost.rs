// Source: /data/home/swei/claudecode/openclaudecode/src/commands/cost/cost.ts
use super::Command;

pub fn create_cost_command() -> Command {
    Command::local("cost", "Display cost breakdown for the current session")
        .supports_non_interactive(true)
}
