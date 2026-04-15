// Source: /data/home/swei/claudecode/openclaudecode/src/commands/fast/fast.tsx
use super::Command;

pub fn create_fast_command() -> Command {
    Command::prompt("fast", "Quick task mode")
}

pub fn create_feedback_command() -> Command {
    Command::local("feedback", "Send feedback").argument_hint("<message>")
}

pub fn create_chrome_command() -> Command {
    Command::local("chrome", "Open Chrome")
}

pub fn create_heapdump_command() -> Command {
    Command::local("heapdump", "Generate heap dump")
}

pub fn create_teleport_command() -> Command {
    Command::local("teleport", "Teleport to environment").argument_hint("[<environment-id>]")
}
