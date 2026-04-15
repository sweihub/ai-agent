// Source: /data/home/swei/claudecode/openclaudecode/src/commands/voice/voice.ts
use super::Command;

pub fn create_voice_command() -> Command {
    Command::local("voice", "Manage voice input").argument_hint("[on|off|status]")
}
