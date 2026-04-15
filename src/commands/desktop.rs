// Source: /data/home/swei/claudecode/openclaudecode/src/commands/desktop/desktop.tsx
use super::Command;

pub fn create_desktop_command() -> Command {
    Command::local("desktop", "Open desktop app")
}
