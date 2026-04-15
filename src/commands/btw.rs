// Source: /data/home/swei/claudecode/openclaudecode/src/commands/btw/btw.tsx
use super::Command;

pub fn create_btw_command() -> Command {
    Command::local("btw", "Add a by-the-way note to the conversation")
        .argument_hint("<message>")
        .supports_non_interactive(true)
}
