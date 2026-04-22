//! PowerShell execution utilities.
//!
//! This module provides PowerShell command parsing, execution, and security analysis.

use std::process::Command;

/// Escape a string for use in PowerShell
pub fn escape_powershell_string(s: &str) -> String {
    // Replace backticks, double quotes, and $ with escaped versions
    s.replace('`', "``").replace('"', "`\"").replace('$', "`$")
}

// ============================================================================
// PowerShell Parser - AST types and parsing functions
// ============================================================================

use serde::{Deserialize, Serialize};

/// Pipeline element type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PipelineElementType {
    CommandAst,
    CommandExpressionAst,
    ParenExpressionAst,
}

/// Command element type (AST node classification)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandElementType {
    ScriptBlock,
    SubExpression,
    ExpandableString,
    MemberInvocation,
    Variable,
    StringConstant,
    Parameter,
    Other,
}

/// Statement type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatementType {
    PipelineAst,
    PipelineChainAst,
    AssignmentStatementAst,
    IfStatementAst,
    ForStatementAst,
    ForEachStatementAst,
    WhileStatementAst,
    DoWhileStatementAst,
    DoUntilStatementAst,
    SwitchStatementAst,
    TryStatementAst,
    TrapStatementAst,
    FunctionDefinitionAst,
    DataStatementAst,
    UnknownStatementAst,
}

/// A child node of a command element
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandElementChild {
    pub element_type: CommandElementType,
    pub text: String,
}

/// Redirection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedRedirection {
    pub from: String,
    pub to: String,
    pub is_merging: bool,
}

/// A command invocation within a pipeline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedCommandElement {
    pub name: String,
    pub name_type: String,
    pub element_type: PipelineElementType,
    pub args: Vec<String>,
    pub text: String,
    pub element_types: Option<Vec<CommandElementType>>,
    pub children: Option<Vec<Option<Vec<CommandElementChild>>>>,
    pub redirections: Option<Vec<ParsedRedirection>>,
}

/// Pipeline segment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineSegment {
    pub commands: Vec<ParsedCommandElement>,
    pub redirections: Vec<ParsedRedirection>,
    pub nested_commands: Option<Vec<ParsedCommandElement>>,
}

/// A statement in the PowerShell command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedStatement {
    pub statement_type: StatementType,
    pub commands: Vec<ParsedCommandElement>,
}

/// Complete parsed PowerShell command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedPowerShellCommand {
    pub valid: bool,
    pub statements: Vec<ParsedStatement>,
    pub error: Option<String>,
}

/// Check if a string is a PowerShell parameter
pub fn is_powershell_parameter(arg: &str, element_type: Option<&CommandElementType>) -> bool {
    if let Some(et) = element_type {
        return *et == CommandElementType::Parameter;
    }
    // Check for common parameter prefixes
    arg.starts_with('-')
        || arg.starts_with('/')
        || arg.starts_with('–')
        || arg.starts_with('—')
        || arg.starts_with('―')
}

/// Alternative parameter prefix characters
pub const PS_TOKENIZER_DASH_CHARS: &[char] = &['-', '–', '—', '―', '/'];

/// Parse a PowerShell command string into components
pub fn parse_powershell_command(command: &str) -> ParsedPowerShellCommand {
    let trimmed = command.trim();

    if trimmed.is_empty() {
        return ParsedPowerShellCommand {
            valid: false,
            statements: vec![],
            error: Some("Empty command".to_string()),
        };
    }

    // Split by statement separators: ; and newlines
    let statement_strs: Vec<&str> = trimmed
        .split(|c| c == ';' || c == '\n')
        .filter(|s| !s.trim().is_empty())
        .collect();

    let mut statements = Vec::new();

    for stmt_str in statement_strs {
        let statement_type = detect_statement_type(stmt_str);

        // Parse pipeline elements (split by |)
        let pipeline_strs: Vec<&str> = stmt_str.split('|').collect();
        let mut commands = Vec::new();

        for (idx, pipeline_str) in pipeline_strs.iter().enumerate() {
            let pipeline_trimmed = pipeline_str.trim();
            if pipeline_trimmed.is_empty() {
                continue;
            }

            // Split by operators: &, && (but NOT | since we already split by | for pipeline)
            let parts: Vec<&str> = pipeline_trimmed
                .split(|c| c == '&')
                .filter(|s| !s.trim().is_empty())
                .collect();

            for part in parts {
                let part_trimmed = part.trim();
                if part_trimmed.is_empty() {
                    continue;
                }

                let cmd = parse_command_element(part_trimmed, idx == 0);
                commands.push(cmd);
            }
        }

        if !commands.is_empty() {
            statements.push(ParsedStatement {
                statement_type,
                commands,
            });
        }
    }

    ParsedPowerShellCommand {
        valid: !statements.is_empty(),
        statements,
        error: None,
    }
}

/// Detect statement type from keywords
fn detect_statement_type(cmd: &str) -> StatementType {
    let lower = cmd.to_lowercase();

    if lower.contains(" if ") || lower.starts_with("if ") {
        StatementType::IfStatementAst
    } else if lower.contains(" foreach ") || lower.starts_with("foreach ") || lower.contains("%{") {
        StatementType::ForEachStatementAst
    } else if lower.contains(" for ") || lower.starts_with("for ") {
        StatementType::ForStatementAst
    } else if lower.contains(" while ") || lower.starts_with("while ") {
        StatementType::WhileStatementAst
    } else if lower.contains(" do ") || lower.starts_with("do ") {
        StatementType::DoWhileStatementAst
    } else if lower.contains(" switch ") || lower.starts_with("switch ") {
        StatementType::SwitchStatementAst
    } else if lower.contains(" try ") || lower.starts_with("try ") {
        StatementType::TryStatementAst
    } else if lower.contains(" function ") || lower.starts_with("function ") {
        StatementType::FunctionDefinitionAst
    } else if lower.contains('=') && !lower.contains("==") {
        StatementType::AssignmentStatementAst
    } else {
        StatementType::PipelineAst
    }
}

/// Parse a single command element
fn parse_command_element(text: &str, is_first: bool) -> ParsedCommandElement {
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.is_empty() {
        return create_empty_command(text.to_string());
    }

    let name = parts[0].to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    let name_type = classify_command_name(&name);
    let element_type = if is_first {
        PipelineElementType::CommandAst
    } else {
        PipelineElementType::CommandExpressionAst
    };
    let element_types = Some(determine_element_types(&args));

    ParsedCommandElement {
        name,
        name_type,
        element_type,
        args,
        text: text.to_string(),
        element_types,
        children: None,
        redirections: None,
    }
}

/// Create an empty command
fn create_empty_command(text: String) -> ParsedCommandElement {
    ParsedCommandElement {
        name: String::new(),
        name_type: "unknown".to_string(),
        element_type: PipelineElementType::CommandAst,
        args: vec![],
        text,
        element_types: None,
        children: None,
        redirections: None,
    }
}

/// Classify command name type
fn classify_command_name(name: &str) -> String {
    let lower = name.to_lowercase();

    // Check if it's a cmdlet (Verb-Noun pattern)
    if lower.contains('-') {
        return "cmdlet".to_string();
    }

    // Check if it has path separators (application)
    if lower.contains('\\') || lower.contains('/') || lower.contains('.') {
        return "application".to_string();
    }

    // Common external commands
    let external = [
        "git", "gh", "docker", "npm", "node", "python", "make", "tar", "curl", "wget",
    ];
    if external.contains(&lower.as_str()) {
        return "application".to_string();
    }

    "unknown".to_string()
}

/// Determine element types for arguments
fn determine_element_types(args: &[String]) -> Vec<CommandElementType> {
    let mut types = vec![CommandElementType::StringConstant];

    for arg in args {
        let et = classify_argument_element(arg);
        types.push(et);
    }

    types
}

/// Classify an argument's element type
fn classify_argument_element(arg: &str) -> CommandElementType {
    let trimmed = arg.trim();

    // Check for variable $var (includes $_.Name, $env:VAR, etc.)
    if trimmed.starts_with('$') && trimmed.len() > 1 {
        let second = trimmed.chars().nth(1);
        // $() is subexpression, $var is variable
        if second == Some('(') || second == Some('@') {
            return CommandElementType::SubExpression;
        }
        // Check if it's $_.Property (member access with variable)
        if second == Some('_') || second.is_some_and(|c| c.is_alphabetic()) {
            return CommandElementType::Variable;
        }
        return CommandElementType::Variable;
    }

    // Check for script block {} - exact match OR contains script block content
    // Handles: {}, { code }, { $_.Name }, and partial tokens like { and }
    if trimmed.starts_with('{')
        || trimmed.ends_with('}')
        || trimmed.contains("{ ")
        || trimmed.contains(" }")
        || trimmed.contains("{}")
    {
        return CommandElementType::ScriptBlock;
    }

    // Check for subexpression $() - exact match OR contains it
    if trimmed.starts_with("$(")
        || trimmed.starts_with("@(")
        || trimmed.contains("$(")
        || trimmed.contains("@(")
    {
        return CommandElementType::SubExpression;
    }

    // Check for expandable string
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        return CommandElementType::ExpandableString;
    }

    // Check for member invocation .Method()
    if trimmed.contains('.') && trimmed.contains('(') {
        return CommandElementType::MemberInvocation;
    }

    // Check for parameter
    if is_powershell_parameter(trimmed, None) {
        return CommandElementType::Parameter;
    }

    CommandElementType::StringConstant
}

/// Derive security flags from parsed command
pub fn derive_security_flags(parsed: &ParsedPowerShellCommand) -> SecurityFlags {
    let mut flags = SecurityFlags::default();

    for statement in &parsed.statements {
        for cmd in &statement.commands {
            // Check element_types first
            if let Some(ref types) = cmd.element_types {
                for et in types {
                    match et {
                        CommandElementType::ScriptBlock => flags.has_script_blocks = true,
                        CommandElementType::SubExpression => flags.has_sub_expressions = true,
                        CommandElementType::ExpandableString => flags.has_expandable_strings = true,
                        CommandElementType::MemberInvocation => flags.has_member_invocations = true,
                        CommandElementType::Variable => flags.has_variables = true,
                        _ => {}
                    }
                }
            }

            // Also check raw args for variables and script blocks (handles edge cases)
            for arg in &cmd.args {
                // Check for variables: $var, $env:VAR, $_.prop
                if arg.starts_with('$') && arg.len() > 1 {
                    let second = arg.chars().nth(1);
                    // $(), @() are subexpressions
                    if second == Some('(') || second == Some('@') {
                        flags.has_sub_expressions = true;
                    } else {
                        flags.has_variables = true;
                    }
                }
                // Check for script blocks
                if arg.contains('{') || arg.contains('}') {
                    flags.has_script_blocks = true;
                }
                // Check for subexpressions
                if arg.contains("$(") || arg.contains("@(") {
                    flags.has_sub_expressions = true;
                }
                // Check for expandable strings
                if arg.starts_with('"') && arg.ends_with('"') {
                    flags.has_expandable_strings = true;
                }
                // Check for assignments
                if arg.contains('=') && !arg.starts_with('-') {
                    flags.has_assignments = true;
                }
            }

            // Also check cmd.text (the full command text) for edge cases
            // e.g., "$env:SECRET" becomes a command with name="$env:SECRET", args=[]
            let text = &cmd.text;
            if text.starts_with('$') && text.len() > 1 && !text.contains(' ') {
                // Single token like $env:SECRET - it's a variable reference
                flags.has_variables = true;
            }
            // Check for script blocks in text
            if text.contains('{') || text.contains('}') {
                flags.has_script_blocks = true;
            }
            // Check for subexpressions in text
            if text.contains("$(") || text.contains("@(") {
                flags.has_sub_expressions = true;
            }
        }
    }

    flags
}

/// Security flags derived from parsing
#[derive(Debug, Clone, Default)]
pub struct SecurityFlags {
    pub has_script_blocks: bool,
    pub has_sub_expressions: bool,
    pub has_expandable_strings: bool,
    pub has_member_invocations: bool,
    pub has_splatting: bool,
    pub has_assignments: bool,
    pub has_stop_parsing: bool,
    pub has_variables: bool,
}

// ============================================================================
// Shell execution functions
// ============================================================================

/// Build a PowerShell command that outputs as UTF-8
pub fn build_powershell_command(script: &str) -> Command {
    let mut cmd = Command::new("pwsh");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    cmd
}

/// Build a PowerShell command with UTF-8 output encoding
pub fn build_powershell_command_utf8(script: &str) -> Command {
    let full_script = format!(
        "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; {}",
        script
    );
    build_powershell_command(&full_script)
}

/// Check if PowerShell is available
pub fn is_powershell_available() -> bool {
    Command::new("pwsh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the PowerShell version
pub fn get_powershell_version() -> Option<String> {
    Command::new("pwsh")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let result = parse_powershell_command("Get-Content file.txt");
        assert!(result.valid);
        assert_eq!(result.statements.len(), 1);
        assert_eq!(result.statements[0].commands[0].name, "Get-Content");
    }

    #[test]
    fn test_parse_command_with_args() {
        let result = parse_powershell_command("Remove-Item -Path test.txt -Recurse -Force");
        assert!(result.valid);
        let cmd = &result.statements[0].commands[0];
        assert_eq!(cmd.name, "Remove-Item");
        assert!(cmd.args.contains(&"-Path".to_string()));
    }

    #[test]
    fn test_parse_pipeline() {
        let result = parse_powershell_command("Get-Content file.txt | Select-String pattern");
        assert!(result.valid);
        assert_eq!(result.statements[0].commands.len(), 2);
    }

    #[test]
    fn test_parse_compound_statements() {
        let result = parse_powershell_command("$var = 1; Get-Content file.txt");
        assert!(result.valid);
        assert_eq!(result.statements.len(), 2);
    }

    #[test]
    fn test_detect_variables() {
        let result = parse_powershell_command("Write-Host $env:SECRET");
        assert!(result.valid);
        let types = &result.statements[0].commands[0].element_types;
        assert!(
            types
                .as_ref()
                .map(|t| t.iter().any(|et| *et == CommandElementType::Variable))
                .unwrap_or(false)
        );
    }

    #[test]
    fn test_detect_script_blocks() {
        let result = parse_powershell_command("Where-Object { $_.Name }");
        assert!(result.valid);
        let types = &result.statements[0].commands[0].element_types;
        assert!(
            types
                .as_ref()
                .map(|t| t.iter().any(|et| *et == CommandElementType::ScriptBlock))
                .unwrap_or(false)
        );
    }

    #[test]
    fn test_detect_subexpression() {
        let result = parse_powershell_command("Invoke-Expression $(malicious)");
        assert!(result.valid);
        let types = &result.statements[0].commands[0].element_types;
        assert!(
            types
                .as_ref()
                .map(|t| t.iter().any(|et| *et == CommandElementType::SubExpression))
                .unwrap_or(false)
        );
    }

    #[test]
    fn test_classify_cmdlet() {
        assert_eq!(classify_command_name("Get-Content"), "cmdlet");
        assert_eq!(classify_command_name("Remove-Item"), "cmdlet");
    }

    #[test]
    fn test_classify_application() {
        assert_eq!(classify_command_name("git"), "application");
        assert_eq!(classify_command_name("./script.ps1"), "application");
    }

    #[test]
    fn test_is_powershell_parameter() {
        assert!(is_powershell_parameter("-Path", None));
        assert!(is_powershell_parameter("-Recurse", None));
        assert!(is_powershell_parameter("/C", None));
        assert!(!is_powershell_parameter("file.txt", None));
    }

    #[test]
    fn test_derive_security_flags_variables() {
        let parsed = parse_powershell_command("$env:SECRET | Write-Host");
        let flags = derive_security_flags(&parsed);
        assert!(flags.has_variables);
    }

    #[test]
    fn test_derive_security_flags_script_blocks() {
        let parsed = parse_powershell_command("Get-Process | Where-Object { $_.CPU }");
        let flags = derive_security_flags(&parsed);
        assert!(flags.has_script_blocks);
    }

    #[test]
    fn test_derive_security_flags_subexpression() {
        let parsed = parse_powershell_command("Invoke-Expression $(malicious)");
        let flags = derive_security_flags(&parsed);
        assert!(flags.has_sub_expressions);
    }

    #[test]
    fn test_derive_security_flags_assignment() {
        let parsed = parse_powershell_command("$result = Get-Content file.txt");
        let flags = derive_security_flags(&parsed);
        assert!(flags.has_assignments);
    }

    #[test]
    fn test_empty_command() {
        let result = parse_powershell_command("");
        assert!(!result.valid);
    }

    #[test]
    fn test_member_invocation() {
        let result = parse_powershell_command("$obj.Method()");
        assert!(result.valid);
    }

    #[test]
    fn test_parse_alias() {
        let result = parse_powershell_command("gc file.txt");
        assert!(result.valid);
        assert_eq!(result.statements[0].commands[0].name, "gc");
    }
}
