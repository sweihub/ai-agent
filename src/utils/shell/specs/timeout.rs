// Source: ~/claudecode/openclaudecode/src/utils/bash/specs/timeout.ts

use super::alias::{CommandArg, CommandSpec};

/// Command specification for the `timeout` command.
pub fn timeout_spec() -> CommandSpec {
    CommandSpec {
        name: "timeout".to_string(),
        description: "Run a command with a time limit".to_string(),
        options: Vec::new(),
        args: vec![
            CommandArg {
                name: "duration".to_string(),
                description: "Duration to wait before timing out (e.g., 10, 5s, 2m)".to_string(),
                is_optional: false,
                is_variadic: false,
                is_command: false,
            },
            CommandArg {
                name: "command".to_string(),
                description: "Command to run".to_string(),
                is_optional: false,
                is_variadic: false,
                is_command: true,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_spec() {
        let spec = timeout_spec();
        assert_eq!(spec.name, "timeout");
        assert_eq!(spec.args.len(), 2);
        assert!(!spec.args[0].is_command);
        assert!(spec.args[1].is_command);
    }
}
