// Source: /data/home/swei/claudecode/openclaudecode/src/commands/branch/branch.ts
use super::Command;

pub fn create_branch_command() -> Command {
    Command::local(
        "branch",
        "Create a branch of the current conversation at this point",
    )
    .argument_hint("[<name>]")
}

pub fn create_fork_command() -> Command {
    Command::local("fork", "Fork the current conversation").argument_hint("[<name>]")
}
