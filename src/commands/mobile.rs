// Source: /data/home/swei/claudecode/openclaudecode/src/commands/mobile/mobile.tsx
use super::Command;

pub fn create_mobile_command() -> Command {
    Command::local("mobile", "Manage mobile app").argument_hint("[pair|unpair|status]")
}

pub fn create_vim_command() -> Command {
    Command::local("vim", "Toggle Vim mode")
}

pub fn create_voice_command() -> Command {
    Command::local("voice", "Manage voice input").argument_hint("[on|off|status]")
}

pub fn create_bridge_command() -> Command {
    Command::local("bridge", "Bridge to other sessions")
        .argument_hint("[connect|disconnect] [<session-id>]")
}

pub fn create_bridge_kick_command() -> Command {
    Command::local("bridge-kick", "Kick from bridged session").argument_hint("<session-id>")
}
