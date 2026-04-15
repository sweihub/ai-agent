use super::Command;

pub fn create_init_verifiers_command() -> Command {
    Command::prompt(
        "init-verifiers",
        "Create verifier skill(s) for automated verification of code changes",
    )
}
