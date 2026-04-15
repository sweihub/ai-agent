// Source: ~/claudecode/openclaudecode/src/utils/bash/specs/pyright.ts

use super::alias::{CommandArg, CommandOption, CommandSpec};

/// Command specification for the `pyright` command (Python type checker).
pub fn pyright_spec() -> CommandSpec {
    CommandSpec {
        name: "pyright".to_string(),
        description: "Type checker for Python".to_string(),
        options: vec![
            CommandOption {
                name: vec!["--help".to_string(), "-h".to_string()],
                description: "Show help message".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--version".to_string()],
                description: "Print pyright version and exit".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--watch".to_string(), "-w".to_string()],
                description: "Continue to run and watch for changes".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--project".to_string(), "-p".to_string()],
                description: "Use the configuration file at this location".to_string(),
                args: Some(CommandArg {
                    name: "FILE OR DIRECTORY".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["-".to_string()],
                description: "Read file or directory list from stdin".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--createstub".to_string()],
                description: "Create type stub file(s) for import".to_string(),
                args: Some(CommandArg {
                    name: "IMPORT".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["--typeshedpath".to_string(), "-t".to_string()],
                description: "Use typeshed type stubs at this location".to_string(),
                args: Some(CommandArg {
                    name: "DIRECTORY".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["--verifytypes".to_string()],
                description: "Verify completeness of types in py.typed package".to_string(),
                args: Some(CommandArg {
                    name: "IMPORT".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["--ignoreexternal".to_string()],
                description: "Ignore external imports for --verifytypes".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--pythonpath".to_string()],
                description: "Path to the Python interpreter".to_string(),
                args: Some(CommandArg {
                    name: "FILE".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["--pythonplatform".to_string()],
                description: "Analyze for platform".to_string(),
                args: Some(CommandArg {
                    name: "PLATFORM".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["--pythonversion".to_string()],
                description: "Analyze for Python version".to_string(),
                args: Some(CommandArg {
                    name: "VERSION".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["--venvpath".to_string(), "-v".to_string()],
                description: "Directory that contains virtual environments".to_string(),
                args: Some(CommandArg {
                    name: "DIRECTORY".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["--outputjson".to_string()],
                description: "Output results in JSON format".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--verbose".to_string()],
                description: "Emit verbose diagnostics".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--stats".to_string()],
                description: "Print detailed performance stats".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--dependencies".to_string()],
                description: "Emit import dependency information".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--level".to_string()],
                description: "Minimum diagnostic level".to_string(),
                args: Some(CommandArg {
                    name: "LEVEL".to_string(),
                    description: String::new(),
                    is_optional: false,
                    is_variadic: false,
                    is_command: false,
                }),
            },
            CommandOption {
                name: vec!["--skipunannotated".to_string()],
                description: "Skip type analysis of unannotated functions".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--warnings".to_string()],
                description: "Use exit code of 1 if warnings are reported".to_string(),
                args: None,
            },
            CommandOption {
                name: vec!["--threads".to_string()],
                description: "Use up to N threads to parallelize type checking".to_string(),
                args: Some(CommandArg {
                    name: "N".to_string(),
                    description: String::new(),
                    is_optional: true,
                    is_variadic: false,
                    is_command: false,
                }),
            },
        ],
        args: vec![CommandArg {
            name: "files".to_string(),
            description: "Specify files or directories to analyze (overrides config file)".to_string(),
            is_optional: true,
            is_variadic: true,
            is_command: false,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pyright_spec() {
        let spec = pyright_spec();
        assert_eq!(spec.name, "pyright");
        assert!(!spec.options.is_empty());
    }
}
