// Source: /data/home/swei/claudecode/openclaudecode/src/commands/rename/rename.ts
use super::Command;

pub fn create_rename_command() -> Command {
    Command::local("rename", "Rename session").argument_hint("[<name>]")
}
