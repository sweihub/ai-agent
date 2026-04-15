// Source: /data/home/swei/claudecode/openclaudecode/src/commands/permissions/permissions.tsx
use super::Command;

pub fn create_permissions_command() -> Command {
    Command::local("permissions", "Manage permissions").argument_hint("[grant|revoke] [<tool>]")
}
