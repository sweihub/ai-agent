// Source: /data/home/swei/claudecode/openclaudecode/src/commands/stickers/stickers.ts
use super::Command;

pub fn create_stickers_command() -> Command {
    Command::local("stickers", "Manage stickers").argument_hint("[list|add|use] [<sticker-name>]")
}

pub fn create_rewind_command() -> Command {
    Command::local("rewind", "Rewind conversation").argument_hint("[<steps>]")
}

pub fn create_thinkback_command() -> Command {
    Command::local("thinkback", "Think about past context").argument_hint("<prompt>")
}

pub fn create_thinkback_play_command() -> Command {
    Command::local("thinkback-play", "Play thinkback session").argument_hint("<session-id>")
}

pub fn create_release_notes_command() -> Command {
    Command::local("release-notes", "Show release notes")
}
