// Source: /data/home/swei/claudecode/openclaudecode/src/commands/upgrade/upgrade.tsx
use super::Command;

pub fn create_upgrade_command() -> Command {
    Command::local("upgrade", "Upgrade Claude Code")
}
