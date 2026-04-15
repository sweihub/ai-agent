// Source: /data/home/swei/claudecode/openclaudecode/src/utils/env.ts
use super::Command;

pub fn create_env_command() -> Command {
    Command::local("env", "Show environment")
}
