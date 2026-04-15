use super::Command;

pub fn create_add_dir_command() -> Command {
    Command::local("add-dir", "Add a new working directory").argument_hint("<path>")
}
