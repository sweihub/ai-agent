// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/built-in/claudeCodeGuideAgent.ts
#![allow(dead_code)]

use std::sync::Arc;

use super::super::AgentDefinition;

const BASH_TOOL_NAME: &str = "Bash";
const FILE_READ_TOOL_NAME: &str = "FileRead";
const GLOB_TOOL_NAME: &str = "Glob";
const GREP_TOOL_NAME: &str = "Grep";
const SEND_MESSAGE_TOOL_NAME: &str = "SendMessage";
const WEB_FETCH_TOOL_NAME: &str = "WebFetch";
const WEB_SEARCH_TOOL_NAME: &str = "WebSearch";

const CLAUDE_CODE_DOCS_MAP_URL: &str = "https://code.claude.com/docs/en/claude_code_docs_map.md";
const CDP_DOCS_MAP_URL: &str = "https://platform.claude.com/llms.txt";

pub const CLAUDE_CODE_GUIDE_AGENT_TYPE: &str = "claude-code-guide";

/// Check if embedded search tools are available.
fn has_embedded_search_tools() -> bool {
    std::env::var("AI_CODE_EMBEDDED_SEARCH_TOOLS")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

/// Check if using 3P services.
fn is_using_3p_services() -> bool {
    std::env::var("AI_CODE_3P_SERVICES")
        .map(|v| v == "1" || v == "true" || v == "yes")
        .unwrap_or(false)
}

pub fn get_claude_code_guide_base_prompt() -> String {
    let local_search_hint = if has_embedded_search_tools() {
        format!("{}, `find`, and `grep`", FILE_READ_TOOL_NAME)
    } else {
        format!(
            "{}, {}, and {}",
            FILE_READ_TOOL_NAME, GLOB_TOOL_NAME, GREP_TOOL_NAME
        )
    };

    format!(
        r#"You are the Claude guide agent. Your primary responsibility is helping users understand and use Claude Code, the Claude Agent SDK, and the Claude API (formerly the Anthropic API) effectively.

**Your expertise spans three domains:**

1. **Claude Code** (the CLI tool): Installation, configuration, hooks, skills, MCP servers, keyboard shortcuts, IDE integrations, settings, and workflows.

2. **Claude Agent SDK**: A framework for building custom AI agents based on Claude Code technology. Available for Node.js/TypeScript and Python.

3. **Claude API**: The Claude API (formerly known as the Anthropic API) for direct model interaction, tool use, and integrations.

**Documentation sources:**

- **Claude Code docs** ({claude_docs}): Fetch this for questions about the Claude Code CLI tool, including:
  - Installation, setup, and getting started
  - Hooks (pre/post command execution)
  - Custom skills
  - MCP server configuration
  - IDE integrations (VS Code, JetBrains)
  - Settings files and configuration
  - Keyboard shortcuts and hotkeys
  - Subagents and plugins
  - Sandboxing and security

- **Claude Agent SDK docs** ({cdp_docs}): Fetch this for questions about building agents with the SDK, including:
  - SDK overview and getting started (Python and TypeScript)
  - Agent configuration + custom tools
  - Session management and permissions
  - MCP integration in agents
  - Hosting and deployment
  - Cost tracking and context management
  Note: Agent SDK docs are part of the Claude API documentation at the same URL.

- **Claude API docs** ({cdp_docs}): Fetch this for questions about the Claude API (formerly the Anthropic API), including:
  - Messages API and streaming
  - Tool use (function calling) and Anthropic-defined tools (computer use, code execution, web search, text editor, bash, programmatic tool calling, tool search tool, context editing, Files API, structured outputs)
  - Vision, PDF support, and citations
  - Extended thinking and structured outputs
  - MCP connector for remote MCP servers
  - Cloud provider integrations (Bedrock, Vertex AI, Foundry)

**Approach:**
1. Determine which domain the user's question falls into
2. Use {web_fetch} to fetch the appropriate docs map
3. Identify the most relevant documentation URLs from the map
4. Fetch the specific documentation pages
5. Provide clear, actionable guidance based on official documentation
6. Use {web_search} if docs don't cover the topic
7. Reference local project files (CLAUDE.md, .claude/ directory) when relevant using {local_search}

**Guidelines:**
- Always prioritize official documentation over assumptions
- Keep responses concise and actionable
- Include specific examples or code snippets when helpful
- Reference exact documentation URLs in your responses
- Help users discover features by proactively suggesting related commands, shortcuts, or capabilities

Complete the user's request by providing accurate, documentation-based guidance."#,
        claude_docs = CLAUDE_CODE_DOCS_MAP_URL,
        cdp_docs = CDP_DOCS_MAP_URL,
        web_fetch = WEB_FETCH_TOOL_NAME,
        web_search = WEB_SEARCH_TOOL_NAME,
        local_search = local_search_hint,
    )
}

pub fn get_feedback_guideline() -> String {
    if is_using_3p_services() {
        "- When you cannot find an answer or the feature doesn't exist, direct the user to file an issue via the appropriate feedback channel".to_string()
    } else {
        "- When you cannot find an answer or the feature doesn't exist, direct the user to use /feedback to report a feature request or bug".to_string()
    }
}

/// Get the tools available to the Claude Code Guide agent.
pub fn get_claude_code_guide_tools() -> Vec<String> {
    if has_embedded_search_tools() {
        vec![
            BASH_TOOL_NAME.to_string(),
            FILE_READ_TOOL_NAME.to_string(),
            WEB_FETCH_TOOL_NAME.to_string(),
            WEB_SEARCH_TOOL_NAME.to_string(),
        ]
    } else {
        vec![
            GLOB_TOOL_NAME.to_string(),
            GREP_TOOL_NAME.to_string(),
            FILE_READ_TOOL_NAME.to_string(),
            WEB_FETCH_TOOL_NAME.to_string(),
            WEB_SEARCH_TOOL_NAME.to_string(),
        ]
    }
}

/// Build context sections for the system prompt from the tool context.
fn build_context_sections(
    custom_commands: &[(String, String)],
    custom_agents: &[(String, String)],
    mcp_clients: &[String],
    plugin_commands: &[(String, String)],
    settings_json: Option<&str>,
) -> String {
    let mut sections: Vec<String> = Vec::new();

    // 1. Custom skills
    if !custom_commands.is_empty() {
        let command_list = custom_commands
            .iter()
            .map(|(name, desc)| format!("- /{}: {}", name, desc))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!(
            "**Available custom skills in this project:**\n{}",
            command_list
        ));
    }

    // 2. Custom agents
    if !custom_agents.is_empty() {
        let agent_list = custom_agents
            .iter()
            .map(|(name, desc)| format!("- {}: {}", name, desc))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!(
            "**Available custom agents configured:**\n{}",
            agent_list
        ));
    }

    // 3. MCP servers
    if !mcp_clients.is_empty() {
        let mcp_list = mcp_clients
            .iter()
            .map(|name| format!("- {}", name))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("**Configured MCP servers:**\n{}", mcp_list));
    }

    // 4. Plugin commands
    if !plugin_commands.is_empty() {
        let plugin_list = plugin_commands
            .iter()
            .map(|(name, desc)| format!("- /{}: {}", name, desc))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("**Available plugin skills:**\n{}", plugin_list));
    }

    // 5. User settings
    if let Some(settings) = settings_json {
        sections.push(format!(
            "**User's settings.json:**\n```json\n{}\n```",
            settings
        ));
    }

    sections.join("\n\n")
}

pub fn claude_code_guide_agent() -> AgentDefinition {
    let tools = get_claude_code_guide_tools();

    AgentDefinition {
        agent_type: CLAUDE_CODE_GUIDE_AGENT_TYPE.to_string(),
        when_to_use: format!(
            "Use this agent when the user asks questions (\"Can Claude...\", \"Does Claude...\", \"How do I...\") about: \
             (1) Claude Code (the CLI tool) - features, hooks, slash commands, MCP servers, settings, IDE integrations, \
             keyboard shortcuts; (2) Claude Agent SDK - building custom agents; (3) Claude API (formerly Anthropic API) - \
             API usage, tool use, Anthropic SDK usage. **IMPORTANT:** Before spawning a new agent, check if there is already \
             a running or recently completed claude-code-guide agent that you can continue via {}.",
            SEND_MESSAGE_TOOL_NAME
        ),
        tools,
        disallowed_tools: vec![],
        source: "built-in".to_string(),
        base_dir: "built-in".to_string(),
        model: Some("haiku".to_string()),
        permission_mode: Some("dontAsk".to_string()),
        max_turns: None,
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
        get_system_prompt: Arc::new(|| {
            let base_prompt = get_claude_code_guide_base_prompt();
            let feedback = get_feedback_guideline();
            format!("{}\n{}", base_prompt, feedback)
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guide_agent_built_in() {
        let agent = claude_code_guide_agent();
        assert_eq!(agent.agent_type, CLAUDE_CODE_GUIDE_AGENT_TYPE);
        assert_eq!(agent.source, "built-in");
        assert_eq!(agent.model, Some("haiku".to_string()));
    }

    #[test]
    fn test_guide_tools_include_web() {
        let agent = claude_code_guide_agent();
        assert!(agent.tools.contains(&"WebFetch".to_string()));
        assert!(agent.tools.contains(&"WebSearch".to_string()));
    }

    #[test]
    fn test_base_prompt_contains_doc_urls() {
        let prompt = get_claude_code_guide_base_prompt();
        assert!(prompt.contains(CLAUDE_CODE_DOCS_MAP_URL));
        assert!(prompt.contains(CDP_DOCS_MAP_URL));
    }
}
