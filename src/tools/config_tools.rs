// Source: ~/claudecode/openclaudecode/src/constants/tools.ts
use crate::utils::env_utils;
use std::collections::HashSet;

// Tool name constants (matching the TypeScript constants/tools.ts)
pub const TASK_OUTPUT_TOOL_NAME: &str = "TaskOutput";
pub const EXIT_PLAN_MODE_V2_TOOL_NAME: &str = "ExitPlanModeV2";
pub const ENTER_PLAN_MODE_TOOL_NAME: &str = "EnterPlanMode";
pub const AGENT_TOOL_NAME: &str = "Agent";
pub const ASK_USER_QUESTION_TOOL_NAME: &str = "AskUserQuestion";
pub const TASK_STOP_TOOL_NAME: &str = "TaskStop";
pub const FILE_READ_TOOL_NAME: &str = "FileRead";
pub const WEB_SEARCH_TOOL_NAME: &str = "WebSearch";
pub const TODO_WRITE_TOOL_NAME: &str = "TodoWrite";
pub const GREP_TOOL_NAME: &str = "Grep";
pub const WEB_FETCH_TOOL_NAME: &str = "WebFetch";
pub const GLOB_TOOL_NAME: &str = "Glob";
pub const FILE_EDIT_TOOL_NAME: &str = "FileEdit";
pub const FILE_WRITE_TOOL_NAME: &str = "FileWrite";
pub const NOTEBOOK_EDIT_TOOL_NAME: &str = "NotebookEdit";
pub const SKILL_TOOL_NAME: &str = "Skill";
pub const SEND_MESSAGE_TOOL_NAME: &str = "SendMessage";
pub const TASK_CREATE_TOOL_NAME: &str = "TaskCreate";
pub const TASK_GET_TOOL_NAME: &str = "TaskGet";
pub const TASK_LIST_TOOL_NAME: &str = "TaskList";
pub const TASK_UPDATE_TOOL_NAME: &str = "TaskUpdate";
pub const TOOL_SEARCH_TOOL_NAME: &str = "ToolSearch";
pub const SYNTHETIC_OUTPUT_TOOL_NAME: &str = "SyntheticOutput";
pub const ENTER_WORKTREE_TOOL_NAME: &str = "EnterWorktree";
pub const EXIT_WORKTREE_TOOL_NAME: &str = "ExitWorktree";
pub const WORKFLOW_TOOL_NAME: &str = "workflow";
pub const CRON_CREATE_TOOL_NAME: &str = "CronCreate";
pub const CRON_DELETE_TOOL_NAME: &str = "CronDelete";
pub const CRON_LIST_TOOL_NAME: &str = "CronList";
pub const REPL_TOOL_NAME: &str = "REPL";
pub const TUNGSTEN_TOOL_NAME: &str = "tungsten";
pub const CONFIG_TOOL_NAME: &str = "Config";
pub const LSP_TOOL_NAME: &str = "LSP";
pub const LIST_MCP_RESOURCES_TOOL_NAME: &str = "ListMcpResourcesTool";
pub const READ_MCP_RESOURCE_TOOL_NAME: &str = "ReadMcpResourceTool";
pub const BASH_TOOL_NAME: &str = "Bash";
pub const POWERSHELL_TOOL_NAME: &str = "PowerShell";
pub const MONITOR_TOOL_NAME: &str = "Monitor";
pub const SEND_USER_FILE_TOOL_NAME: &str = "send_user_file";
pub const WEB_BROWSER_TOOL_NAME: &str = "WebBrowser";
pub const SLEEP_TOOL_NAME: &str = "Sleep";
pub const REMOTE_TRIGGER_TOOL_NAME: &str = "RemoteTrigger";
pub const PUSH_NOTIFICATION_TOOL_NAME: &str = "PushNotification";
pub const SUBSCRIBE_PR_TOOL_NAME: &str = "SubscribePR";
pub const SUGGEST_BACKGROUND_PR_TOOL_NAME: &str = "SuggestBackgroundPR";
pub const OVERFLOW_TEST_TOOL_NAME: &str = "OverflowTestTool";
pub const CTX_INSPECT_TOOL_NAME: &str = "CtxInspect";
pub const TERMINAL_CAPTURE_TOOL_NAME: &str = "terminal_capture";
pub const SNIP_TOOL_NAME: &str = "snip";
pub const VERIFY_PLAN_EXECUTION_TOOL_NAME: &str = "verify_plan_execution";
pub const LIST_PEERS_TOOL_NAME: &str = "ListPeers";

/// Tools disallowed for all agents
pub fn all_agent_disallowed_tools() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert(TASK_OUTPUT_TOOL_NAME);
    set.insert(EXIT_PLAN_MODE_V2_TOOL_NAME);
    set.insert(ENTER_PLAN_MODE_TOOL_NAME);
    // Allow Agent tool for agents when user is ant
    if !env_utils::is_ant_user() {
        set.insert(AGENT_TOOL_NAME);
    }
    set.insert(ASK_USER_QUESTION_TOOL_NAME);
    set.insert(TASK_STOP_TOOL_NAME);
    // Prevent recursive workflow execution inside subagents
    set.insert(WORKFLOW_TOOL_NAME);
    set
}

/// Tools disallowed for custom agents (same as all agents)
pub fn custom_agent_disallowed_tools() -> HashSet<&'static str> {
    all_agent_disallowed_tools()
}

/// Tools allowed for async agents
pub fn async_agent_allowed_tools() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert(FILE_READ_TOOL_NAME);
    set.insert(WEB_SEARCH_TOOL_NAME);
    set.insert(TODO_WRITE_TOOL_NAME);
    set.insert(GREP_TOOL_NAME);
    set.insert(WEB_FETCH_TOOL_NAME);
    set.insert(GLOB_TOOL_NAME);
    // Shell tool names
    set.insert(BASH_TOOL_NAME);
    set.insert(POWERSHELL_TOOL_NAME);
    set.insert(FILE_EDIT_TOOL_NAME);
    set.insert(FILE_WRITE_TOOL_NAME);
    set.insert(NOTEBOOK_EDIT_TOOL_NAME);
    set.insert(SKILL_TOOL_NAME);
    set.insert(SYNTHETIC_OUTPUT_TOOL_NAME);
    set.insert(TOOL_SEARCH_TOOL_NAME);
    set.insert(ENTER_WORKTREE_TOOL_NAME);
    set.insert(EXIT_WORKTREE_TOOL_NAME);
    set
}

/// Tools allowed in coordinator mode
pub fn coordinator_mode_allowed_tools() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert(AGENT_TOOL_NAME);
    set.insert(TASK_STOP_TOOL_NAME);
    set.insert(SEND_MESSAGE_TOOL_NAME);
    set.insert(SYNTHETIC_OUTPUT_TOOL_NAME);
    set
}

/// Tools allowed for in-process teammates
pub fn in_process_teammate_allowed_tools() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert(TASK_CREATE_TOOL_NAME);
    set.insert(TASK_GET_TOOL_NAME);
    set.insert(TASK_LIST_TOOL_NAME);
    set.insert(TASK_UPDATE_TOOL_NAME);
    set.insert(SEND_MESSAGE_TOOL_NAME);
    // Cron tools for teammates
    set.insert(CRON_CREATE_TOOL_NAME);
    set.insert(CRON_DELETE_TOOL_NAME);
    set.insert(CRON_LIST_TOOL_NAME);
    set
}

/// REPL-only tools (hidden from direct use when REPL mode is enabled)
pub fn repl_only_tools() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    set.insert(FILE_READ_TOOL_NAME);
    set.insert(FILE_WRITE_TOOL_NAME);
    set.insert(FILE_EDIT_TOOL_NAME);
    set.insert(GLOB_TOOL_NAME);
    set.insert(GREP_TOOL_NAME);
    set.insert(BASH_TOOL_NAME);
    set.insert(NOTEBOOK_EDIT_TOOL_NAME);
    set.insert(AGENT_TOOL_NAME);
    set
}

/// Check if REPL mode is enabled
pub fn is_repl_mode_enabled() -> bool {
    // If explicitly set to falsy, disable
    if env_utils::is_env_defined_falsy(std::env::var("CLAUDE_CODE_REPL").ok().as_deref()) {
        return false;
    }
    // Legacy env
    if env_utils::is_env_truthy(std::env::var("CLAUDE_REPL_MODE").ok().as_deref()) {
        return true;
    }
    // Ant default for CLI entrypoint
    env_utils::is_ant_user()
        && std::env::var("CLAUDE_CODE_ENTRYPOINT")
            .map(|v| v == "cli")
            .unwrap_or(false)
}

/// Check if tool search is enabled (optimistic check)
pub fn is_tool_search_enabled_optimistic() -> bool {
    let mode = get_tool_search_mode();
    if mode == "standard" {
        return false;
    }
    // Check if using a proxy that might not support tool_reference
    if std::env::var("ENABLE_TOOL_SEARCH").is_err() {
        // If ANTHROPIC_BASE_URL is set but not a first-party host, disable
        if let Ok(base_url) = std::env::var("ANTHROPIC_BASE_URL") {
            let first_party_hosts = ["api.anthropic.com", "api.anthropic.ai"];
            if !first_party_hosts.iter().any(|h| base_url.contains(h)) {
                return false;
            }
        }
    }
    true
}

/// Get tool search mode
pub fn get_tool_search_mode() -> &'static str {
    // Check kill switch
    if env_utils::is_env_truthy(
        std::env::var("CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS")
            .ok()
            .as_deref(),
    ) {
        return "standard";
    }

    let value = std::env::var("ENABLE_TOOL_SEARCH").ok();

    // Handle auto:N syntax
    if let Some(ref v) = value {
        if let Some(percent) = parse_auto_percentage(v) {
            if percent == 0 {
                return "tst";
            }
            if percent == 100 {
                return "standard";
            }
            return "tst-auto";
        }
    }

    if env_utils::is_env_truthy(value.as_deref()) {
        return "tst";
    }
    if env_utils::is_env_defined_falsy(value.as_deref()) {
        return "standard";
    }
    // Default: always defer MCP and shouldDefer tools
    "tst"
}

/// Parse auto:N percentage from ENABLE_TOOL_SEARCH
fn parse_auto_percentage(value: &str) -> Option<i32> {
    if !value.starts_with("auto:") {
        return None;
    }
    let percent_str = &value[5..];
    percent_str.parse::<i32>().ok().map(|p| p.max(0).min(100))
}

/// Check if worktree mode is enabled (unconditionally enabled)
pub fn is_worktree_mode_enabled() -> bool {
    true
}

/// Check if agent swarms (teams) is enabled
pub fn is_agent_swarms_enabled() -> bool {
    // Ant: always on
    if env_utils::is_ant_user() {
        return true;
    }
    // External: require opt-in
    if !env_utils::is_env_truthy(
        std::env::var("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS")
            .ok()
            .as_deref(),
    ) && !std::env::args().any(|a| a == "--agent-teams")
    {
        return false;
    }
    // Killswitch - for now, assume enabled (no GrowthBook in Rust)
    true
}

/// Check if PowerShell tool is enabled
pub fn is_powershell_tool_enabled() -> bool {
    if env_utils::get_platform() != "windows" {
        return false;
    }
    if env_utils::is_ant_user() {
        return !env_utils::is_env_defined_falsy(
            std::env::var("CLAUDE_CODE_USE_POWERSHELL_TOOL")
                .ok()
                .as_deref(),
        );
    }
    env_utils::is_env_truthy(
        std::env::var("CLAUDE_CODE_USE_POWERSHELL_TOOL")
            .ok()
            .as_deref(),
    )
}

/// Shell tool names (Bash + PowerShell if enabled)
pub fn shell_tool_names() -> Vec<&'static str> {
    let mut names = vec![BASH_TOOL_NAME];
    if is_powershell_tool_enabled() {
        names.push(POWERSHELL_TOOL_NAME);
    }
    names
}

/// Check if embedded search tools are available
pub fn has_embedded_search_tools() -> bool {
    if !env_utils::is_env_truthy(std::env::var("EMBEDDED_SEARCH_TOOLS").ok().as_deref()) {
        return false;
    }
    let entrypoint = std::env::var("CLAUDE_CODE_ENTRYPOINT").unwrap_or_default();
    !["sdk-ts", "sdk-py", "sdk-cli", "local-agent"].contains(&entrypoint.as_str())
}

/// Check if tasks v2 is enabled
pub fn is_todo_v2_enabled() -> bool {
    if env_utils::is_env_truthy(std::env::var("CLAUDE_CODE_ENABLE_TASKS").ok().as_deref()) {
        return true;
    }
    // In non-interactive mode, tasks are disabled by default
    // For Rust SDK, we default to enabled for simplicity
    true
}
