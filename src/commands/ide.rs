// Source: /data/home/swei/claudecode/openclaudecode/src/utils/ide.ts
use super::Command;

pub fn create_ide_command() -> Command {
    Command::local("ide", "Open files in IDE").argument_hint("<file>")
}
