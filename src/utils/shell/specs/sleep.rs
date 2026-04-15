// Source: ~/claudecode/openclaudecode/src/utils/bash/specs/sleep.ts

use super::alias::{CommandArg, CommandSpec};

/// Command specification for the `sleep` command.
pub fn sleep_spec() -> CommandSpec {
    CommandSpec {
        name: "sleep".to_string(),
        description: "Delay for a specified amount of time".to_string(),
        options: Vec::new(),
        args: vec![CommandArg {
            name: "duration".to_string(),
            description: "Duration to sleep (seconds or with suffix like 5s, 2m, 1h)".to_string(),
            is_optional: false,
            is_variadic: false,
            is_command: false,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sleep_spec() {
        let spec = sleep_spec();
        assert_eq!(spec.name, "sleep");
        assert_eq!(spec.args.len(), 1);
        assert_eq!(spec.args[0].name, "duration");
    }
}
