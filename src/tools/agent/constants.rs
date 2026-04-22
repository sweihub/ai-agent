// Source: ~/claudecode/openclaudecode/src/tools/AgentTool/constants.ts
pub const AGENT_TOOL_NAME: &str = "Agent";
/// Legacy wire name for backward compat (permission rules, hooks, resumed sessions)
pub const LEGACY_AGENT_TOOL_NAME: &str = "Task";
pub const VERIFICATION_AGENT_TYPE: &str = "verification";

/// Built-in agents that run once and return a report -- the parent never
/// SendMessages back to continue them. Skip the agentId/SendMessage/usage
/// trailer for these to save tokens (~135 chars x 34M Explore runs/week).
pub fn one_shot_builtin_agent_types() -> &'static [&'static str] {
    &["Explore", "Plan"]
}

/// Tools that are disallowed for ALL agents (built-in and custom).
pub const ALL_AGENT_DISALLOWED_TOOLS: &[&str] = &["Skill", "ReloadPlugin", "Prompt"];

/// Tools that are disallowed for non-built-in (custom) agents.
pub const CUSTOM_AGENT_DISALLOWED_TOOLS: &[&str] = &["Skill"];

/// Tools that async (background) agents are allowed to use.
pub const ASYNC_AGENT_ALLOWED_TOOLS: &[&str] = &[
    "Bash",
    "FileRead",
    "FileWrite",
    "FileEdit",
    "Glob",
    "Grep",
    "WebFetch",
    "WebSearch",
    "TodoWrite",
    "TaskCreate",
    "TaskGet",
    "TaskList",
    "TaskUpdate",
    "NotebookEdit",
    "Agent",
];

/// Fork subagent boilerplate XML tag
pub const FORK_BOILERPLATE_TAG: &str = "fork_boilerplate";
pub const FORK_DIRECTIVE_PREFIX: &str = "fork_directive:";

/// Tool name constants referenced in the agent prompt
pub const FILE_READ_TOOL_NAME: &str = "FileRead";
pub const FILE_WRITE_TOOL_NAME: &str = "FileWrite";
pub const GLOB_TOOL_NAME: &str = "Glob";
pub const SEND_MESSAGE_TOOL_NAME: &str = "SendMessage";
