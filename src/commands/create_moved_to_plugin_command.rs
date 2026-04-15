use super::Command;

pub fn create_moved_to_plugin_command(
    name: &str,
    description: &str,
    progress_message: &str,
    plugin_name: &str,
    plugin_command: &str,
) -> Command {
    Command::prompt(name, description).argument_hint(progress_message)
}
