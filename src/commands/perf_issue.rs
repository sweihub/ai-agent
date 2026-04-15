use super::Command;

pub fn create_perf_issue_command() -> Command {
    Command::local("perf-issue", "Report performance issue").argument_hint("<description>")
}
