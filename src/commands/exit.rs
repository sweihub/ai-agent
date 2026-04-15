// Source: /data/home/swei/claudecode/openclaudecode/src/cli/exit.ts
use super::Command;

pub fn create_exit_command() -> Command {
    Command::local("exit", "Exit the current session").argument_hint("[--force]")
}

pub fn create_quit_command() -> Command {
    Command::local("quit", "Exit the current session").argument_hint("[--force]")
}
