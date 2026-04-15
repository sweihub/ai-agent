// Source: ~/claudecode/openclaudecode/src/utils/bash/specs/time.ts

use super::alias::{CommandArg, CommandSpec};

/// Command specification for the `time` command.
pub fn time_spec() -> CommandSpec {
    CommandSpec {
        name: "time".to_string(),
        description: "Time a command".to_string(),
        options: Vec::new(),
        args: vec![CommandArg {
            name: "command".to_string(),
            description: "Command to time".to_string(),
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
    fn test_time_spec() {
        let spec = time_spec();
        assert_eq!(spec.name, "time");
        assert!(spec.args[0].is_command);
    }
}
