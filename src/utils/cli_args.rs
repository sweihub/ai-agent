// Source: ~/claudecode/openclaudecode/src/utils/cliArgs.ts
//! Parse a CLI flag value early, before Commander.js processes arguments.
//! Supports both space-separated (--flag value) and equals-separated (--flag=value) syntax.
//!
//! This function is intended for flags that must be parsed before init() runs,
//! such as --settings which affects configuration loading. For normal flag parsing,
//! rely on Commander.js which handles this automatically.

#![allow(dead_code)]

/// Parse a CLI flag value early, before Commander.js processes arguments.
/// Supports both space-separated (--flag value) and equals-separated (--flag=value) syntax.
///
/// # Arguments
/// * `flag_name` - The flag name including dashes (e.g. '--settings')
/// * `argv` - Optional argv array to parse (defaults to std::env::args())
///
/// # Returns
/// The value if found, None otherwise
pub fn eager_parse_cli_flag(flag_name: &str, argv: Option<&[String]>) -> Option<String> {
    let argv_vec: Vec<String>;
    let args = match argv {
        Some(a) => a,
        None => {
            argv_vec = std::env::args().collect();
            &argv_vec
        }
    };

    for i in 0..args.len() {
        let arg = &args[i];
        // Handle --flag=value syntax
        if arg.starts_with(&format!("{flag_name}=")) {
            return Some(arg[flag_name.len() + 1..].to_string());
        }
        // Handle --flag value syntax
        if arg == flag_name && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
    }
    None
}

/// Handle the standard Unix `--` separator convention in CLI arguments.
///
/// When using Commander.js with `.pass_through_options()`, the `--` separator
/// is passed through as a positional argument rather than being consumed.
/// This means when a user runs:
///   `cmd --opt value name -- subcmd --flag arg`
///
/// Commander parses it as:
///   positional1 = "name", positional2 = "--", rest = ["subcmd", "--flag", "arg"]
///
/// This function corrects the parsing by extracting the actual command from
/// the rest array when the positional is `--`.
///
/// # Arguments
/// * `command_or_value` - The parsed positional that may be "--"
/// * `args` - The remaining arguments array
///
/// # Returns
/// Tuple of (corrected command, remaining args)
pub fn extract_args_after_double_dash(
    command_or_value: &str,
    args: Option<&[String]>,
) -> (String, Vec<String>) {
    let args_vec: Vec<String>;
    let args_slice = match args {
        Some(a) => a,
        None => {
            args_vec = Vec::new();
            &args_vec
        }
    };

    if command_or_value == "--" && !args_slice.is_empty() {
        let command = args_slice[0].clone();
        let remaining = args_slice[1..].to_vec();
        return (command, remaining);
    }
    (command_or_value.to_string(), args_slice.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eager_parse_cli_flag_equals() {
        let argv = vec!["prog".to_string(), "--flag=value".to_string()];
        assert_eq!(
            eager_parse_cli_flag("--flag", Some(&argv)),
            Some("value".to_string())
        );
    }

    #[test]
    fn test_eager_parse_cli_flag_space() {
        let argv = vec![
            "prog".to_string(),
            "--flag".to_string(),
            "value".to_string(),
        ];
        assert_eq!(
            eager_parse_cli_flag("--flag", Some(&argv)),
            Some("value".to_string())
        );
    }

    #[test]
    fn test_eager_parse_cli_flag_not_found() {
        let argv = vec!["prog".to_string(), "--other".to_string()];
        assert_eq!(eager_parse_cli_flag("--flag", Some(&argv)), None);
    }

    #[test]
    fn test_extract_args_after_double_dash() {
        let args = vec![
            "subcmd".to_string(),
            "--flag".to_string(),
            "arg".to_string(),
        ];
        let (command, remaining) = extract_args_after_double_dash("--", Some(&args));
        assert_eq!(command, "subcmd");
        assert_eq!(remaining, vec!["--flag", "arg"]);
    }

    #[test]
    fn test_extract_args_no_double_dash() {
        let args = vec!["--flag".to_string(), "arg".to_string()];
        let (command, remaining) = extract_args_after_double_dash("name", Some(&args));
        assert_eq!(command, "name");
        assert_eq!(remaining, vec!["--flag", "arg"]);
    }

    #[test]
    fn test_extract_args_empty_args() {
        let (command, remaining) = extract_args_after_double_dash("--", None);
        assert_eq!(command, "--");
        assert!(remaining.is_empty());
    }
}
