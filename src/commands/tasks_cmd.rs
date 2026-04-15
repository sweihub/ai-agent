use super::Command;

pub fn create_tasks_command() -> Command {
    Command::local("tasks", "List and manage tasks").argument_hint("[<task-id>]")
}

pub fn create_plan_command() -> Command {
    Command::prompt("plan", "Create a plan for the current task")
}

pub fn create_rewind_command() -> Command {
    Command::local("rewind", "Rewind the conversation to a specific point").argument_hint("<steps>")
}
