// Source: /data/home/swei/claudecode/openclaudecode/src/commands/memory/memory.tsx
use super::Command;

pub fn create_memory_command() -> Command {
    Command::local("memory", "Manage memory").argument_hint("[on|off|clear]")
}
