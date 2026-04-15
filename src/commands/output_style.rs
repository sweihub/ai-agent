use super::Command;

pub fn create_output_style_command() -> Command {
    Command::local(
        "output-style",
        "Deprecated: use /config to change output style",
    )
    .is_hidden(true)
}
