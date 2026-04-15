use super::Command;

pub fn create_oauth_refresh_command() -> Command {
    Command::local("oauth-refresh", "Refresh OAuth token")
}
