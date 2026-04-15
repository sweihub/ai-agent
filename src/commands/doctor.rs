// Source: /data/home/swei/claudecode/openclaudecode/src/commands/doctor/doctor.tsx
use super::Command;

pub fn create_doctor_command() -> Command {
    Command::local("doctor", "Diagnose issues")
}

pub fn create_sandbox_toggle_command() -> Command {
    Command::local("sandbox-toggle", "Toggle sandbox mode")
}

pub fn create_remote_env_command() -> Command {
    Command::local("remote-env", "Manage remote environment")
}

pub fn create_remote_setup_command() -> Command {
    Command::local("remote-setup", "Setup remote access")
}

pub fn create_pr_comments_command() -> Command {
    Command::local("pr-comments", "Manage PR comments")
}
