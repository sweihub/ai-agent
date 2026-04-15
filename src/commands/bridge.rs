// Source: /data/home/swei/claudecode/openclaudecode/src/commands/bridge/bridge.tsx
use super::Command;

pub fn create_bridge_command() -> Command {
    Command::local("bridge", "Bridge to other sessions")
        .argument_hint("[connect|disconnect] [<session-id>]")
}
