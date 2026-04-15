use super::Command;

pub fn create_install_github_app_command() -> Command {
    Command::local(
        "install-github-app",
        "Set up Claude GitHub Actions for a repository",
    )
}
