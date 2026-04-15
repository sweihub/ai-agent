use super::Command;

pub fn create_privacy_settings_command() -> Command {
    Command::local("privacy-settings", "Configure privacy settings")
}
