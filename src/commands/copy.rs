// Source: /data/home/swei/claudecode/openclaudecode/src/commands/copy/copy.tsx
use super::Command;

pub fn create_copy_command() -> Command {
    Command::local("copy", "Copy to clipboard").argument_hint("<text>")
}
