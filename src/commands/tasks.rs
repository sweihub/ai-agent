// Source: /data/home/swei/claudecode/openclaudecode/src/tasks.ts
use super::Command;

pub fn create_tasks_command() -> Command {
    Command::local("tasks", "Manage background tasks")
        .argument_hint("[list|kill|resume] [<task-id>]")
}

pub fn create_status_command() -> Command {
    Command::local("status", "Show session status")
}

pub fn create_stats_command() -> Command {
    Command::local("stats", "Show usage statistics")
}

pub fn create_theme_command() -> Command {
    Command::local("theme", "Change the theme").argument_hint("[theme-name]")
}

pub fn create_tag_command() -> Command {
    Command::local("tag", "Tag this session").argument_hint("[<tag-name>]")
}
