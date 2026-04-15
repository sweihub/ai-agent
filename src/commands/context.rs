// Source: /data/home/swei/claudecode/openclaudecode/src/context.ts
use super::Command;

pub fn create_context_command() -> Command {
    Command::local("context", "Show context usage")
}
