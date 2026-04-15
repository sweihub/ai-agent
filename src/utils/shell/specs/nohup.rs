// Source: ~/claudecode/openclaudecode/src/utils/bash/specs/nohup.ts

use super::alias::{CommandArg, CommandSpec};

/// Command specification for the `nohup` command.
pub fn nohup_spec() -> CommandSpec {
    CommandSpec {
        name: "nohup".to_string(),
        description: "Run a command immune to hangups".to_string(),
        options: Vec::new(),
        args: vec![CommandArg {
            name: "command".to_string(),
            description: "Command to run with nohup".to_string(),
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
    fn test_nohup_spec() {
        let spec = nohup_spec();
        assert_eq!(spec.name, "nohup");
        assert_eq!(spec.args.len(), 1);
        assert!(spec.args[0].is_command);
    }
}
