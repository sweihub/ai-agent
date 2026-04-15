// Source: /data/home/swei/claudecode/openclaudecode/src/commands/help/help.tsx
use super::Command;

pub fn create_help_command() -> Command {
    Command::local("help", "Show available commands")
        .argument_hint("[<command>]")
        .supports_non_interactive(true)
}

pub fn create_whoami_command() -> Command {
    Command::local("whoami", "Display current user information").supports_non_interactive(true)
}
