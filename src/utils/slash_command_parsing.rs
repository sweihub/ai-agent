// Source: ~/claudecode/openclaudecode/src/utils/slashCommandParsing.rs

/// A parsed slash command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSlashCommand {
    pub command_name: String,
    pub args: String,
    pub is_mcp: bool,
}

/// Parses a slash command input string into its component parts.
///
/// # Examples
///
/// ```
/// use ai_agent::utils::slash_command_parsing::parse_slash_command;
///
/// let result = parse_slash_command("/search foo bar").unwrap();
/// assert_eq!(result.command_name, "search");
/// assert_eq!(result.args, "foo bar");
/// assert!(!result.is_mcp);
///
/// let result = parse_slash_command("/mcp:tool (MCP) arg1 arg2").unwrap();
/// assert_eq!(result.command_name, "mcp:tool (MCP)");
/// assert_eq!(result.args, "arg1 arg2");
/// assert!(result.is_mcp);
/// ```
pub fn parse_slash_command(input: &str) -> Option<ParsedSlashCommand> {
    let trimmed_input = input.trim();

    // Check if input starts with '/'
    if !trimmed_input.starts_with('/') {
        return None;
    }

    // Remove the leading '/' and split by spaces
    let without_slash = &trimmed_input[1..];
    let words: Vec<&str> = without_slash.splitn(2, ' ').collect();

    if words.is_empty() || words[0].is_empty() {
        return None;
    }

    let command_name;
    let is_mcp;
    let args;

    // Check the rest for MCP pattern
    if let Some(rest) = words.get(1) {
        // Check if second word is '(MCP)'
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();
        if parts.first() == Some(&"(MCP)") {
            command_name = format!("{} (MCP)", words[0]);
            is_mcp = true;
            args = parts.get(1).unwrap_or(&"").to_string();
        } else {
            command_name = words[0].to_string();
            is_mcp = false;
            args = rest.to_string();
        }
    } else {
        command_name = words[0].to_string();
        is_mcp = false;
        args = String::new();
    }

    Some(ParsedSlashCommand {
        command_name,
        args,
        is_mcp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let result = parse_slash_command("/search foo bar").unwrap();
        assert_eq!(result.command_name, "search");
        assert_eq!(result.args, "foo bar");
        assert!(!result.is_mcp);
    }

    #[test]
    fn test_parse_mcp_command() {
        let result = parse_slash_command("/mcp:tool (MCP) arg1 arg2").unwrap();
        assert_eq!(result.command_name, "mcp:tool (MCP)");
        assert_eq!(result.args, "arg1 arg2");
        assert!(result.is_mcp);
    }

    #[test]
    fn test_parse_no_leading_slash() {
        assert!(parse_slash_command("search foo").is_none());
    }

    #[test]
    fn test_parse_empty_command() {
        assert!(parse_slash_command("/").is_none());
        assert!(parse_slash_command("/ ").is_none());
    }

    #[test]
    fn test_parse_command_no_args() {
        let result = parse_slash_command("/search").unwrap();
        assert_eq!(result.command_name, "search");
        assert_eq!(result.args, "");
    }
}
