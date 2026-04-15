#![allow(dead_code)]

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CodeIndexingTool {
    Sourcegraph,
    Hound,
    Seagoat,
    Bloop,
    Gitloop,
    Cody,
    Aider,
    Continue,
    GitHubCopilot,
    Cursor,
    Tabby,
    Codeium,
    Tabnine,
    Augment,
    Windsurf,
    Aide,
    Pieces,
    Qodo,
    AmazonQ,
    Gemini,
    ClaudeContext,
    CodeIndexMcp,
    LocalCodeSearch,
    AutodevCodebase,
    OpenCtx,
}

impl std::fmt::Display for CodeIndexingTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeIndexingTool::Sourcegraph => write!(f, "sourcegraph"),
            CodeIndexingTool::Hound => write!(f, "hound"),
            CodeIndexingTool::Seagoat => write!(f, "seagoat"),
            CodeIndexingTool::Bloop => write!(f, "bloop"),
            CodeIndexingTool::Gitloop => write!(f, "gitloop"),
            CodeIndexingTool::Cody => write!(f, "cody"),
            CodeIndexingTool::Aider => write!(f, "aider"),
            CodeIndexingTool::Continue => write!(f, "continue"),
            CodeIndexingTool::GitHubCopilot => write!(f, "github-copilot"),
            CodeIndexingTool::Cursor => write!(f, "cursor"),
            CodeIndexingTool::Tabby => write!(f, "tabby"),
            CodeIndexingTool::Codeium => write!(f, "codeium"),
            CodeIndexingTool::Tabnine => write!(f, "tabnine"),
            CodeIndexingTool::Augment => write!(f, "augment"),
            CodeIndexingTool::Windsurf => write!(f, "windsurf"),
            CodeIndexingTool::Aide => write!(f, "aide"),
            CodeIndexingTool::Pieces => write!(f, "pieces"),
            CodeIndexingTool::Qodo => write!(f, "qodo"),
            CodeIndexingTool::AmazonQ => write!(f, "amazon-q"),
            CodeIndexingTool::Gemini => write!(f, "gemini"),
            CodeIndexingTool::ClaudeContext => write!(f, "claude-context"),
            CodeIndexingTool::CodeIndexMcp => write!(f, "code-index-mcp"),
            CodeIndexingTool::LocalCodeSearch => write!(f, "local-code-search"),
            CodeIndexingTool::AutodevCodebase => write!(f, "autodev-codebase"),
            CodeIndexingTool::OpenCtx => write!(f, "openctx"),
        }
    }
}

lazy_static! {
    static ref CLI_COMMAND_MAPPING: HashMap<String, CodeIndexingTool> = {
        let mut m = HashMap::new();
        m.insert("src".to_string(), CodeIndexingTool::Sourcegraph);
        m.insert("cody".to_string(), CodeIndexingTool::Cody);
        m.insert("aider".to_string(), CodeIndexingTool::Aider);
        m.insert("tabby".to_string(), CodeIndexingTool::Tabby);
        m.insert("tabnine".to_string(), CodeIndexingTool::Tabnine);
        m.insert("augment".to_string(), CodeIndexingTool::Augment);
        m.insert("pieces".to_string(), CodeIndexingTool::Pieces);
        m.insert("qodo".to_string(), CodeIndexingTool::Qodo);
        m.insert("aide".to_string(), CodeIndexingTool::Aide);
        m.insert("hound".to_string(), CodeIndexingTool::Hound);
        m.insert("seagoat".to_string(), CodeIndexingTool::Seagoat);
        m.insert("bloop".to_string(), CodeIndexingTool::Bloop);
        m.insert("gitloop".to_string(), CodeIndexingTool::Gitloop);
        m.insert("q".to_string(), CodeIndexingTool::AmazonQ);
        m.insert("gemini".to_string(), CodeIndexingTool::Gemini);
        m
    };
    static ref MCP_SERVER_PATTERNS: Vec<(Regex, CodeIndexingTool)> = vec![
        (
            Regex::new(r"(?i)^sourcegraph$").unwrap(),
            CodeIndexingTool::Sourcegraph
        ),
        (Regex::new(r"(?i)^cody$").unwrap(), CodeIndexingTool::Cody),
        (
            Regex::new(r"(?i)^openctx$").unwrap(),
            CodeIndexingTool::OpenCtx
        ),
        (Regex::new(r"(?i)^aider$").unwrap(), CodeIndexingTool::Aider),
        (
            Regex::new(r"(?i)^continue$").unwrap(),
            CodeIndexingTool::Continue
        ),
        (
            Regex::new(r"(?i)^github[-_]?copilot$").unwrap(),
            CodeIndexingTool::GitHubCopilot
        ),
        (
            Regex::new(r"(?i)^copilot$").unwrap(),
            CodeIndexingTool::GitHubCopilot
        ),
        (
            Regex::new(r"(?i)^cursor$").unwrap(),
            CodeIndexingTool::Cursor
        ),
        (Regex::new(r"(?i)^tabby$").unwrap(), CodeIndexingTool::Tabby),
        (
            Regex::new(r"(?i)^codeium$").unwrap(),
            CodeIndexingTool::Codeium
        ),
        (
            Regex::new(r"(?i)^tabnine$").unwrap(),
            CodeIndexingTool::Tabnine
        ),
        (
            Regex::new(r"(?i)^augment[-_]?code$").unwrap(),
            CodeIndexingTool::Augment
        ),
        (
            Regex::new(r"(?i)^augment$").unwrap(),
            CodeIndexingTool::Augment
        ),
        (
            Regex::new(r"(?i)^windsurf$").unwrap(),
            CodeIndexingTool::Windsurf
        ),
        (Regex::new(r"(?i)^aide$").unwrap(), CodeIndexingTool::Aide),
        (
            Regex::new(r"(?i)^codestory$").unwrap(),
            CodeIndexingTool::Aide
        ),
        (
            Regex::new(r"(?i)^pieces$").unwrap(),
            CodeIndexingTool::Pieces
        ),
        (Regex::new(r"(?i)^qodo$").unwrap(), CodeIndexingTool::Qodo),
        (
            Regex::new(r"(?i)^amazon[-_]?q$").unwrap(),
            CodeIndexingTool::AmazonQ
        ),
        (
            Regex::new(r"(?i)^gemini[-_]?code[-_]?assist$").unwrap(),
            CodeIndexingTool::Gemini
        ),
        (
            Regex::new(r"(?i)^gemini$").unwrap(),
            CodeIndexingTool::Gemini
        ),
        (Regex::new(r"(?i)^hound$").unwrap(), CodeIndexingTool::Hound),
        (
            Regex::new(r"(?i)^seagoat$").unwrap(),
            CodeIndexingTool::Seagoat
        ),
        (Regex::new(r"(?i)^bloop$").unwrap(), CodeIndexingTool::Bloop),
        (
            Regex::new(r"(?i)^gitloop$").unwrap(),
            CodeIndexingTool::Gitloop
        ),
        (
            Regex::new(r"(?i)^claude[-_]?context$").unwrap(),
            CodeIndexingTool::ClaudeContext
        ),
        (
            Regex::new(r"(?i)^code[-_]?index[-_]?mcp$").unwrap(),
            CodeIndexingTool::CodeIndexMcp
        ),
        (
            Regex::new(r"(?i)^code[-_]?index$").unwrap(),
            CodeIndexingTool::CodeIndexMcp
        ),
        (
            Regex::new(r"(?i)^local[-_]?code[-_]?search$").unwrap(),
            CodeIndexingTool::LocalCodeSearch
        ),
        (
            Regex::new(r"(?i)^codebase$").unwrap(),
            CodeIndexingTool::AutodevCodebase
        ),
        (
            Regex::new(r"(?i)^autodev[-_]?codebase$").unwrap(),
            CodeIndexingTool::AutodevCodebase
        ),
        (
            Regex::new(r"(?i)^code[-_]?context$").unwrap(),
            CodeIndexingTool::ClaudeContext
        ),
    ];
}

pub fn detect_code_indexing_from_command(command: &str) -> Option<CodeIndexingTool> {
    let trimmed = command.trim();
    let first_word = trimmed.split_whitespace().next()?.to_lowercase();

    if first_word == "npx" || first_word == "bunx" {
        let second_word = trimmed.split_whitespace().nth(1)?.to_lowercase();
        return CLI_COMMAND_MAPPING.get(&second_word).cloned();
    }

    CLI_COMMAND_MAPPING.get(&first_word).cloned()
}

pub fn detect_code_indexing_from_mcp_tool(tool_name: &str) -> Option<CodeIndexingTool> {
    if !tool_name.starts_with("mcp__") {
        return None;
    }

    let parts: Vec<&str> = tool_name.split("__").collect();
    if parts.len() < 3 {
        return None;
    }

    let server_name = parts.get(1)?;
    if server_name.is_empty() {
        return None;
    }

    for (pattern, tool) in MCP_SERVER_PATTERNS.iter() {
        if pattern.is_match(server_name) {
            return Some(tool.clone());
        }
    }

    None
}

pub fn detect_code_indexing_from_mcp_server_name(server_name: &str) -> Option<CodeIndexingTool> {
    for (pattern, tool) in MCP_SERVER_PATTERNS.iter() {
        if pattern.is_match(server_name) {
            return Some(tool.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_command() {
        assert_eq!(
            detect_code_indexing_from_command("src search \"pattern\""),
            Some(CodeIndexingTool::Sourcegraph)
        );
        assert_eq!(
            detect_code_indexing_from_command("cody chat --message \"help\""),
            Some(CodeIndexingTool::Cody)
        );
        assert_eq!(detect_code_indexing_from_command("ls -la"), None);
    }

    #[test]
    fn test_detect_from_mcp_tool() {
        assert_eq!(
            detect_code_indexing_from_mcp_tool("mcp__sourcegraph__search"),
            Some(CodeIndexingTool::Sourcegraph)
        );
        assert_eq!(
            detect_code_indexing_from_mcp_tool("mcp__cody__chat"),
            Some(CodeIndexingTool::Cody)
        );
        assert_eq!(
            detect_code_indexing_from_mcp_tool("mcp__filesystem__read"),
            None
        );
    }

    #[test]
    fn test_detect_from_mcp_server_name() {
        assert_eq!(
            detect_code_indexing_from_mcp_server_name("sourcegraph"),
            Some(CodeIndexingTool::Sourcegraph)
        );
        assert_eq!(
            detect_code_indexing_from_mcp_server_name("filesystem"),
            None
        );
    }
}
