// Source: /data/home/swei/claudecode/openclaudecode/src/commands/plan/plan.tsx
use super::Command;

pub fn create_plan_command() -> Command {
    Command::local("plan", "Manage plan").argument_hint("[create|edit|approve|reject] [<plan-id>]")
}
