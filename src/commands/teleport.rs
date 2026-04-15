// Source: /data/home/swei/claudecode/openclaudecode/src/utils/teleport.tsx
use super::Command;

pub fn create_teleport_command() -> Command {
    Command::local("teleport", "Teleport to environment").argument_hint("[<environment-id>]")
}
