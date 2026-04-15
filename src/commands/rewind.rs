// Source: /data/home/swei/claudecode/openclaudecode/src/commands/rewind/rewind.ts
use super::Command;

pub fn create_rewind_command() -> Command {
    Command::local("rewind", "Rewind conversation").argument_hint("[<steps>]")
}
