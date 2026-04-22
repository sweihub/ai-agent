// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/built-in/exploreAgent.ts
#![allow(dead_code)]
use std::sync::Arc;

use super::super::AgentDefinition;

const BASH_TOOL_NAME: &str = "Bash";
const EXIT_PLAN_MODE_TOOL_NAME: &str = "ExitPlanMode";
const FILE_EDIT_TOOL_NAME: &str = "FileEdit";
const FILE_READ_TOOL_NAME: &str = "FileRead";
const FILE_WRITE_TOOL_NAME: &str = "FileWrite";
const GLOB_TOOL_NAME: &str = "Glob";
const GREP_TOOL_NAME: &str = "Grep";
const NOTEBOOK_EDIT_TOOL_NAME: &str = "NotebookEdit";
const AGENT_TOOL_NAME: &str = "Agent";

/// Check if embedded search tools are available.
pub fn has_embedded_search_tools() -> bool {
    std::env::var("AI_CODE_EMBEDDED_SEARCH_TOOLS")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

pub fn get_explore_system_prompt() -> String {
    let embedded = has_embedded_search_tools();
    let glob_guidance = if embedded {
        format!(
            "- Use `find` via {} for broad file pattern matching",
            BASH_TOOL_NAME
        )
    } else {
        format!("- Use {} for broad file pattern matching", GLOB_TOOL_NAME)
    };
    let grep_guidance = if embedded {
        format!(
            "- Use `grep` via {} for searching file contents with regex",
            BASH_TOOL_NAME
        )
    } else {
        format!(
            "- Use {} for searching file contents with regex",
            GREP_TOOL_NAME
        )
    };

    let embedded_note = if embedded { ", grep" } else { "" };

    format!(
        r#"You are a file search specialist for Claude Code, Anthropic's official CLI for Claude. You excel at thoroughly navigating and exploring codebases.

=== CRITICAL: READ-ONLY MODE - NO FILE MODIFICATIONS ===
This is a READ-ONLY exploration task. You are STRICTLY PROHIBITED from:
- Creating new files (no Write, touch, or file creation of any kind)
- Modifying existing files (no Edit operations)
- Deleting files (no rm or deletion)
- Moving or copying files (no mv or cp)
- Creating temporary files anywhere, including /tmp
- Using redirect operators (>, >>, |) or heredocs to write to files
- Running ANY commands that change system state

Your role is EXCLUSIVELY to search and analyze existing code. You do NOT have access to file editing tools - attempting to edit files will fail.

Your strengths:
- Rapidly finding files using glob patterns
- Searching code and text with powerful regex patterns
- Reading and analyzing file contents

Guidelines:
{glob_guidance}
{grep_guidance}
- Use {file_read} when you know the specific file path you need to read
- Use {bash} ONLY for read-only operations (ls, git status, git log, git diff, find{embedded_note}, cat, head, tail)
- NEVER use {bash} for: mkdir, touch, rm, cp, mv, git add, git commit, npm install, pip install, or any file creation/modification
- Adapt your search approach based on the thoroughness level specified by the caller
- Communicate your final report directly as a regular message - do NOT attempt to create files

NOTE: You are meant to be a fast agent that returns output as quickly as possible. In order to achieve this you must:
- Make efficient use of the tools that you have at your disposal: be smart about how you search for files and implementations
- Wherever possible you should try to spawn multiple parallel tool calls for grepping and reading files

Complete the user's search request efficiently and report your findings clearly."#,
        file_read = FILE_READ_TOOL_NAME,
        bash = BASH_TOOL_NAME,
        embedded_note = embedded_note,
    )
}

pub const EXPLORE_AGENT_MIN_QUERIES: usize = 3;

const EXPLORE_WHEN_TO_USE: &str = "Fast agent specialized for exploring codebases. Use this when you need to quickly find files by patterns (eg. \"src/components/**/*.tsx\"), search code for keywords (eg. \"API endpoints\"), or answer questions about the codebase (eg. \"how do API endpoints work?\"). When calling this agent, specify the desired thoroughness level: \"quick\" for basic searches, \"medium\" for moderate exploration, or \"very thorough\" for comprehensive analysis across multiple locations and naming conventions.";

pub fn explore_agent() -> AgentDefinition {
    let user_type = std::env::var("AI_CODE_USER_TYPE").unwrap_or_default();

    AgentDefinition {
        agent_type: "Explore".to_string(),
        when_to_use: EXPLORE_WHEN_TO_USE.to_string(),
        disallowed_tools: vec![
            AGENT_TOOL_NAME.to_string(),
            EXIT_PLAN_MODE_TOOL_NAME.to_string(),
            FILE_EDIT_TOOL_NAME.to_string(),
            FILE_WRITE_TOOL_NAME.to_string(),
            NOTEBOOK_EDIT_TOOL_NAME.to_string(),
        ],
        tools: vec!["*".to_string()],
        source: "built-in".to_string(),
        base_dir: "built-in".to_string(),
        // Ants get inherit to use the main agent's model; external users get haiku for speed
        model: if user_type == "ant" {
            Some("inherit".to_string())
        } else {
            Some("haiku".to_string())
        },
        max_turns: None,
        permission_mode: None,
        effort: None,
        color: None,
        mcp_servers: vec![],
        hooks: None,
        skills: vec![],
        background: false,
        initial_prompt: None,
        memory: None,
        isolation: None,
        required_mcp_servers: vec![],
        omit_claude_md: true,
        critical_system_reminder_experimental: None,
        get_system_prompt: Arc::new(get_explore_system_prompt),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explore_agent_built_in() {
        let agent = explore_agent();
        assert_eq!(agent.agent_type, "Explore");
        assert_eq!(agent.source, "built-in");
        assert!(agent.omit_claude_md);
    }

    #[test]
    fn test_explore_agent_disallowed_tools() {
        let agent = explore_agent();
        assert!(agent.disallowed_tools.contains(&"FileWrite".to_string()));
        assert!(agent.disallowed_tools.contains(&"FileEdit".to_string()));
    }

    #[test]
    fn test_explore_system_prompt_contains_readonly() {
        let prompt = get_explore_system_prompt();
        assert!(prompt.contains("READ-ONLY"));
        assert!(prompt.contains("STRICTLY PROHIBITED"));
    }
}
