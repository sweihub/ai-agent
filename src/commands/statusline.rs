//! Statusline command
//! Translated from: ~/claudecode/openclaudecode/src/commands/statusline.tsx

use anyhow::Result;

pub fn execute_statusline_command(args: &str) -> Result<String> {
    let prompt = if args.trim().is_empty() {
        "Configure my statusLine from my shell PS1 configuration"
    } else {
        args
    };

    // This creates an agent to configure the statusline
    Ok(format!(
        "Creating statusline setup agent with prompt: \"{}\"",
        prompt
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statusline_command_empty() {
        let result = execute_statusline_command("").unwrap();
        assert!(result.contains("statusline setup agent"));
    }

    #[test]
    fn test_statusline_command_with_args() {
        let result = execute_statusline_command("custom prompt").unwrap();
        assert!(result.contains("custom prompt"));
    }
}
