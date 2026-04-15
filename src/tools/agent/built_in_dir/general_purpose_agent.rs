// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/built-in/generalPurposeAgent.ts
use super::super::AgentDefinition;
use std::sync::Arc;

const SHARED_PREFIX: &str = "You are an agent for Claude Code, Anthropic's official CLI for Claude. Given the user's message, you should use the tools available to complete the task. Complete the task fully--don't gold-plate, but don't leave it half-done.";

const SHARED_GUIDELINES: &str = r#"Your strengths:
- Searching for code, configurations, and patterns across large codebases
- Analyzing multiple files to understand system architecture
- Investigating complex questions that require exploring many files
- Performing multi-step research tasks

Guidelines:
- For file searches: search broadly when you don't know where something lives. Use Read when you know the specific file path.
- For analysis: Start broad and narrow down. Use multiple search strategies if the first doesn't yield results.
- Be thorough: Check multiple locations, consider different naming conventions, look for related files.
- NEVER create files unless they're absolutely necessary for achieving your goal. ALWAYS prefer editing an existing file to creating a new one.
- NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested."#;

fn get_general_purpose_system_prompt() -> String {
    format!(
        "{SHARED_PREFIX} When you complete the task, respond with a concise report covering what was done and any key findings -- the caller will relay this to the user, so it only needs the essentials.\n\n{SHARED_GUIDELINES}"
    )
}

pub fn general_purpose_agent() -> AgentDefinition {
    AgentDefinition {
        agent_type: "general-purpose".to_string(),
        when_to_use: "General-purpose agent for researching complex questions, searching for code, and executing multi-step tasks. When you are searching for a keyword or file and are not confident that you will find the right match in the first few tries use this agent to perform the search for you.".to_string(),
        tools: vec!["*".to_string()],
        disallowed_tools: vec![],
        source: "built-in".to_string(),
        base_dir: "built-in".to_string(),
        get_system_prompt: Arc::new(get_general_purpose_system_prompt),
        model: Some("sonnet".to_string()),
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
        omit_claude_md: false,
        critical_system_reminder_experimental: None,
    }
}
