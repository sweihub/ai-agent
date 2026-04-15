use super::Command;

pub fn create_install_slack_app_command() -> Command {
    Command::local("install-slack-app", "Install the Claude Slack app")
        .supports_non_interactive(false)
}
