// Source: /data/home/swei/claudecode/openclaudecode/src/commands/thinkback/thinkback.tsx
use super::Command;

pub fn create_thinkback_command() -> Command {
    Command::local("thinkback", "Think about past context").argument_hint("<prompt>")
}
