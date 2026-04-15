// Source: ~/claudecode/openclaudecode/src/utils/bash/specs/alias.ts

/// Command specification for the `alias` command.
pub fn alias_spec() -> CommandSpec {
    CommandSpec {
        name: "alias".to_string(),
        description: "Create or list command aliases".to_string(),
        options: Vec::new(),
        args: vec![CommandArg {
            name: "definition".to_string(),
            description: "Alias definition in the form name=value".to_string(),
            is_optional: true,
            is_variadic: true,
            is_command: false,
        }],
    }
}

/// Command specification structure.
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub name: String,
    pub description: String,
    pub options: Vec<CommandOption>,
    pub args: Vec<CommandArg>,
}

/// Command option specification.
#[derive(Debug, Clone)]
pub struct CommandOption {
    pub name: Vec<String>,
    pub description: String,
    pub args: Option<CommandArg>,
}

/// Command argument specification.
#[derive(Debug, Clone)]
pub struct CommandArg {
    pub name: String,
    pub description: String,
    pub is_optional: bool,
    pub is_variadic: bool,
    pub is_command: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_spec() {
        let spec = alias_spec();
        assert_eq!(spec.name, "alias");
        assert!(!spec.args.is_empty());
    }
}
