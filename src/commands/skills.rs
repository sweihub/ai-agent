// Source: /data/home/swei/claudecode/openclaudecode/src/commands/skills/skills.tsx
use super::{Command, CommandCallResult};

pub fn create_skills_command() -> Command {
    Command::local("skills", "List available skills")
}
