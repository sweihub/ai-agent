// Source: /data/home/swei/claudecode/openclaudecode/src/commands/compact/compact.ts
use super::Command;

pub fn create_compact_command() -> Command {
    Command::local("compact", "Compact the conversation to save space")
}

pub fn create_summarize_command() -> Command {
    Command::local("summarize", "Summarize the current conversation")
}
