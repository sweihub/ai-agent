use super::Command;

pub fn create_bughunter_command() -> Command {
    Command::local("bughunter", "Run bug hunting").argument_hint("[<target>]")
}
