use super::Command;

pub fn create_thinkback_play_command() -> Command {
    Command::local("thinkback-play", "Play thinkback session").argument_hint("<session-id>")
}
