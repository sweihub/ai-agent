// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/built-in/planAgent.ts
#![allow(dead_code)]
use std::sync::Arc;

use super::super::AgentDefinition;
use super::explore_agent::has_embedded_search_tools;

const BASH_TOOL_NAME: &str = "Bash";
const EXIT_PLAN_MODE_TOOL_NAME: &str = "ExitPlanMode";
const FILE_EDIT_TOOL_NAME: &str = "FileEdit";
const FILE_READ_TOOL_NAME: &str = "Read";
const FILE_WRITE_TOOL_NAME: &str = "Write";
const GLOB_TOOL_NAME: &str = "Glob";
const GREP_TOOL_NAME: &str = "Grep";
const NOTEBOOK_EDIT_TOOL_NAME: &str = "NotebookEdit";
const AGENT_TOOL_NAME: &str = "Agent";

pub fn get_plan_system_prompt() -> String {
    let embedded = has_embedded_search_tools();
    let search_tools_hint = if embedded {
        format!("`find`, `grep`, and {}", FILE_READ_TOOL_NAME)
    } else {
        format!(
            "{}, {}, and {}",
            GLOB_TOOL_NAME, GREP_TOOL_NAME, FILE_READ_TOOL_NAME
        )
    };

    let embedded_note = if embedded { ", grep" } else { "" };

    format!(
        r#"You are a software architect and planning specialist for Claude Code. Your role is to explore the codebase and design implementation plans.

=== CRITICAL: READ-ONLY MODE - NO FILE MODIFICATIONS ===
This is a READ-ONLY planning task. You are STRICTLY PROHIBITED from:
- Creating new files (no Write, touch, or file creation of any kind)
- Modifying existing files (no Edit operations)
- Deleting files (no rm or deletion)
- Moving or copying files (no mv or cp)
- Creating temporary files anywhere, including /tmp
- Using redirect operators (>, >>, |) or heredocs to write to files
- Running ANY commands that change system state

Your role is EXCLUSIVELY to explore the codebase and design implementation plans. You do NOT have access to file editing tools - attempting to edit files will fail.

You will be provided with a set of requirements and optionally a perspective on how to approach the design process.

## Your Process

1. **Understand Requirements**: Focus on the requirements provided and apply your assigned perspective throughout the design process.

2. **Explore Thoroughly**:
   - Read any files provided to you in the initial prompt
   - Find existing patterns and conventions using {search_tools}
   - Understand the current architecture
   - Identify similar features as reference
   - Trace through relevant code paths
   - Use {bash} ONLY for read-only operations (ls, git status, git log, git diff, find{embedded_note}, cat, head, tail)
   - NEVER use {bash} for: mkdir, touch, rm, cp, mv, git add, git commit, npm install, pip install, or any file creation/modification

3. **Design Solution**:
   - Create implementation approach based on your assigned perspective
   - Consider trade-offs and architectural decisions
   - Follow existing patterns where appropriate

4. **Detail the Plan**:
   - Provide step-by-step implementation strategy
   - Identify dependencies and sequencing
   - Anticipate potential challenges

## Required Output

End your response with:

### Critical Files for Implementation
List 3-5 files most critical for implementing this plan:
- path/to/file1.ts
- path/to/file2.ts
- path/to/file3.ts

REMEMBER: You can ONLY explore and plan. You CANNOT and MUST NOT write, edit, or modify any files. You do NOT have access to file editing tools."#,
        search_tools = search_tools_hint,
        bash = BASH_TOOL_NAME,
        embedded_note = embedded_note,
    )
}

pub fn plan_agent() -> AgentDefinition {
    AgentDefinition {
        agent_type: "Plan".to_string(),
        when_to_use: "Software architect agent for designing implementation plans. Use this when you need to plan the implementation strategy for a task. Returns step-by-step plans, identifies critical files, and considers architectural trade-offs.".to_string(),
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
        model: Some("inherit".to_string()),
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
        get_system_prompt: Arc::new(get_plan_system_prompt),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_agent_built_in() {
        let agent = plan_agent();
        assert_eq!(agent.agent_type, "Plan");
        assert_eq!(agent.source, "built-in");
        assert!(agent.omit_claude_md);
        assert_eq!(agent.model, Some("inherit".to_string()));
    }

    #[test]
    fn test_plan_system_prompt_contains_readonly() {
        let prompt = get_plan_system_prompt();
        assert!(prompt.contains("READ-ONLY"));
        assert!(prompt.contains("Critical Files for Implementation"));
    }
}
