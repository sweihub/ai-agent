// Source: /data/home/swei/claudecode/openclaudecode/src/commands/clear/clear.ts
use super::Command;

pub fn create_clear_command() -> Command {
    Command::local("clear", "Clear the conversation").argument_hint("[cache|conversation|all]")
}

pub fn create_clear_cache_command() -> Command {
    Command::local("clear-cache", "Clear cached data")
}

pub fn create_clear_conversation_command() -> Command {
    Command::local("clear-conversation", "Clear the current conversation")
}
