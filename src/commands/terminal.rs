// Source: /data/home/swei/claudecode/openclaudecode/src/utils/terminal.ts
use super::Command;

pub fn create_terminal_setup_command() -> Command {
    Command::local("terminal-setup", "Setup terminal integration")
}

pub fn create_desktop_command() -> Command {
    Command::local("desktop", "Open desktop app")
}

pub fn create_ide_command() -> Command {
    Command::local("ide", "Open files in IDE").argument_hint("<file>")
}

pub fn create_keybindings_command() -> Command {
    Command::local("keybindings", "Show keybindings")
}

pub fn create_hooks_command() -> Command {
    Command::local("hooks", "Manage hooks").argument_hint("[list|add|remove] [<hook-name>]")
}
