// Source: /data/home/swei/claudecode/openclaudecode/src/commands/brief.ts
use super::Command;

pub fn create_brief_command() -> Command {
    Command::local("brief", "Toggle brief-only mode")
}

#[derive(Debug, Clone, Default)]
pub struct BriefConfig {
    pub enable_slash_command: bool,
}

pub fn get_default_brief_config() -> BriefConfig {
    BriefConfig {
        enable_slash_command: false,
    }
}
