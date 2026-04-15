// Source: /data/home/swei/claudecode/openclaudecode/src/commands/keybindings/keybindings.ts
use super::Command;

pub fn create_keybindings_command() -> Command {
    Command::local("keybindings", "Show keybindings")
}
