// Source: ~/claudecode/openclaudecode/src/utils/bash/specs/index.ts

use super::alias::CommandSpec;

pub mod alias;
pub mod nohup;
pub mod pyright;
pub mod sleep;
pub mod srun;
pub mod time;
pub mod timeout;

/// Get all command specifications.
pub fn all_specs() -> Vec<CommandSpec> {
    vec![
        pyright::pyright_spec(),
        timeout::timeout_spec(),
        sleep::sleep_spec(),
        alias::alias_spec(),
        nohup::nohup_spec(),
        time::time_spec(),
        srun::srun_spec(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_specs() {
        let specs = all_specs();
        assert_eq!(specs.len(), 7);
    }
}
