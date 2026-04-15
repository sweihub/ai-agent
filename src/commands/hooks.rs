// Source: /data/home/swei/claudecode/openclaudecode/src/commands/hooks/hooks.tsx
use super::Command;

pub fn create_hooks_command() -> Command {
    Command::local("hooks", "Manage hooks").argument_hint("[list|add|remove] [<hook-name>]")
}
