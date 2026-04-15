// Source: /data/home/swei/claudecode/openclaudecode/src/commands/passes/passes.tsx
use super::Command;

pub fn create_passes_command() -> Command {
    Command::local(
        "passes",
        "Share a free week of Claude Code with friends and earn extra usage",
    )
    .is_hidden(true)
}
