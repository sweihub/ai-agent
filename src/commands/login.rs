// Source: /data/home/swei/claudecode/openclaudecode/src/commands/login/login.tsx
use super::Command;

pub fn create_login_command() -> Command {
    Command::local("login", "Login to Claude Code")
}
