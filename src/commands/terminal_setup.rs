use super::Command;

pub fn create_terminal_setup_command() -> Command {
    Command::local(
        "terminal-setup",
        "Install Shift+Enter key binding for newlines",
    )
}
