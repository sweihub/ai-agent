// Source: /data/home/swei/claudecode/openclaudecode/src/commands/diff/diff.tsx
use super::Command;

pub fn create_diff_command() -> Command {
    Command::prompt("diff", "Show the differences between sessions").argument_hint("[<session-id>]")
}

pub fn create_clear_command() -> Command {
    Command::local("clear", "Clear the conversation").argument_hint("[cache|conversation|all]")
}

pub fn create_compact_command() -> Command {
    Command::local("compact", "Compact the conversation to save space")
}

pub fn create_resume_command() -> Command {
    Command::local("resume", "Resume a previous session").argument_hint("<session-id>")
}
