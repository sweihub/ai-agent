// Source: /data/home/swei/claudecode/openclaudecode/src/commands/logout/logout.tsx
use super::Command;

pub fn create_logout_command() -> Command {
    Command::local("logout", "Logout from Claude Code")
}
