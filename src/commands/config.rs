// Source: /data/home/swei/claudecode/openclaudecode/src/services/autoDream/config.ts
use super::Command;

pub fn create_config_command() -> Command {
    Command::local("config", "Manage configuration settings").argument_hint("[<key>] [<value>]")
}

pub fn create_model_command() -> Command {
    Command::local("model", "Switch the AI model").argument_hint("<model-name>")
}

pub fn create_theme_command() -> Command {
    Command::local("theme", "Change the color theme").argument_hint("[<theme-name>]")
}
