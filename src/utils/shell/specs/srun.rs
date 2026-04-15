// Source: ~/claudecode/openclaudecode/src/utils/bash/specs/srun.ts

use super::alias::{CommandArg, CommandOption, CommandSpec};

/// Command specification for the `srun` command (SLURM cluster).
pub fn srun_spec() -> CommandSpec {
    CommandSpec {
        name: "srun".to_string(),
        description: "Run a command on SLURM cluster nodes".to_string(),
        options: vec![
            CommandOption {
                name: vec!["-n".to_string(), "--ntasks".to_string()],
                description: "Number of tasks".to_string(),
                args: Some(CommandArg {
                    name: "count".to_string(),
                    description: "Number of tasks to run".to_string(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["-N".to_string(), "--nodes".to_string()],
                description: "Number of nodes".to_string(),
                args: Some(CommandArg {
                    name: "count".to_string(),
                    description: "Number of nodes to allocate".to_string(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
        ],
        args: vec![CommandArg {
            name: "command".to_string(),
            description: "Command to run on the cluster".to_string(),
            is_optional: false,
            is_variadic: false,
            is_command: true,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srun_spec() {
        let spec = srun_spec();
        assert_eq!(spec.name, "srun");
        assert_eq!(spec.options.len(), 2);
        assert!(spec.args[0].is_command);
    }
}
