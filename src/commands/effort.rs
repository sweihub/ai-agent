// Source: /data/home/swei/claudecode/openclaudecode/src/utils/effort.ts
use super::Command;

pub fn create_effort_command() -> Command {
    Command::local("effort", "Show effort tracking").argument_hint("[update <level>]")
}

pub fn create_plan_command() -> Command {
    Command::local("plan", "Manage plan").argument_hint("[create|edit|approve|reject] [<plan-id>]")
}

pub fn create_permissions_command() -> Command {
    Command::local("permissions", "Manage permissions").argument_hint("[grant|revoke] [<tool>]")
}

pub fn create_resume_command() -> Command {
    Command::local("resume", "Resume a session").argument_hint("<session-id>")
}

pub fn create_upgrade_command() -> Command {
    Command::local("upgrade", "Upgrade Claude Code")
}
