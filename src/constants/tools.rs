// Source: /data/home/swei/claudecode/openclaudecode/src/types/tools.ts
//! Tool constants module
//!
//! Contains sets of allowed and disallowed tools for different agent contexts.

use std::collections::HashSet;

/// Tool name constants - these should match the tool definitions
pub const TASK_OUTPUT_TOOL_NAME: &str = "TaskOutput";
pub const EXIT_PLAN_MODE_V2_TOOL_NAME: &str = "ExitPlanMode";
pub const ENTER_PLAN_MODE_TOOL_NAME: &str = "EnterPlanMode";
pub const AGENT_TOOL_NAME: &str = "Agent";
pub const ASK_USER_QUESTION_TOOL_NAME: &str = "AskUserQuestion";
pub const TASK_STOP_TOOL_NAME: &str = "TaskStop";
pub const FILE_READ_TOOL_NAME: &str = "Read";
pub const WEB_SEARCH_TOOL_NAME: &str = "WebSearch";
pub const TODO_WRITE_TOOL_NAME: &str = "TodoWrite";
pub const GREP_TOOL_NAME: &str = "Grep";
pub const WEB_FETCH_TOOL_NAME: &str = "WebFetch";
pub const GLOB_TOOL_NAME: &str = "Glob";
pub const FILE_EDIT_TOOL_NAME: &str = "Edit";
pub const FILE_WRITE_TOOL_NAME: &str = "Write";
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
pub const WORKFLOW_TOOL_NAME: &str = "Workflow";
pub const CRON_CREATE_TOOL_NAME: &str = "CronCreate";
pub const CRON_DELETE_TOOL_NAME: &str = "CronDelete";
pub const CRON_LIST_TOOL_NAME: &str = "CronList";
pub const BASH_TOOL_NAME: &str = "Bash";
pub const POWERSHELL_TOOL_NAME: &str = "PowerShell";

/// Tools that are disallowed for all agents
pub fn get_all_agent_disallowed_tools() -> HashSet<&'static str> {
    let mut tools = HashSet::new();
    tools.insert(TASK_OUTPUT_TOOL_NAME);
    tools.insert(EXIT_PLAN_MODE_V2_TOOL_NAME);
    tools.insert(ENTER_PLAN_MODE_TOOL_NAME);
    tools.insert(ASK_USER_QUESTION_TOOL_NAME);
    tools.insert(TASK_STOP_TOOL_NAME);
    // Note: AGENT_TOOL_NAME is conditionally added based on USER_TYPE
    // Note: WORKFLOW_TOOL_NAME is conditionally added based on feature flag
    tools
}

/// Tools that are disallowed for custom agents (includes all agent disallowed)
pub fn get_custom_agent_disallowed_tools() -> HashSet<&'static str> {
    get_all_agent_disallowed_tools()
}

/// Tools allowed for async agents
pub fn get_async_agent_allowed_tools() -> HashSet<&'static str> {
    let mut tools = HashSet::new();
    tools.insert(FILE_READ_TOOL_NAME);
    tools.insert(WEB_SEARCH_TOOL_NAME);
    tools.insert(TODO_WRITE_TOOL_NAME);
    tools.insert(GREP_TOOL_NAME);
    tools.insert(WEB_FETCH_TOOL_NAME);
    tools.insert(GLOB_TOOL_NAME);
    tools.insert(BASH_TOOL_NAME);
    tools.insert(POWERSHELL_TOOL_NAME);
    tools.insert(FILE_EDIT_TOOL_NAME);
    tools.insert(FILE_WRITE_TOOL_NAME);
    tools.insert(NOTEBOOK_EDIT_TOOL_NAME);
    tools.insert(SKILL_TOOL_NAME);
    tools.insert(SYNTHETIC_OUTPUT_TOOL_NAME);
    tools.insert(TOOL_SEARCH_TOOL_NAME);
    tools.insert(ENTER_WORKTREE_TOOL_NAME);
    tools.insert(EXIT_WORKTREE_TOOL_NAME);
    tools
}

/// Tools allowed only for in-process teammates
pub fn get_in_process_teammate_allowed_tools() -> HashSet<&'static str> {
    let mut tools = HashSet::new();
    tools.insert(TASK_CREATE_TOOL_NAME);
    tools.insert(TASK_GET_TOOL_NAME);
    tools.insert(TASK_LIST_TOOL_NAME);
    tools.insert(TASK_UPDATE_TOOL_NAME);
    tools.insert(SEND_MESSAGE_TOOL_NAME);
    // Note: CRON tools are conditionally added based on feature flag
    tools
}

/// Tools allowed in coordinator mode
pub fn get_coordinator_mode_allowed_tools() -> HashSet<&'static str> {
    let mut tools = HashSet::new();
    tools.insert(AGENT_TOOL_NAME);
    tools.insert(TASK_STOP_TOOL_NAME);
    tools.insert(SEND_MESSAGE_TOOL_NAME);
    tools.insert(SYNTHETIC_OUTPUT_TOOL_NAME);
    tools
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disallowed_tools() {
        let tools = get_all_agent_disallowed_tools();
        assert!(tools.contains(TASK_OUTPUT_TOOL_NAME));
        assert!(tools.contains(TASK_STOP_TOOL_NAME));
    }

    #[test]
    fn test_async_allowed_tools() {
        let tools = get_async_agent_allowed_tools();
        assert!(tools.contains(FILE_READ_TOOL_NAME));
        assert!(tools.contains(BASH_TOOL_NAME));
        assert!(!tools.contains(AGENT_TOOL_NAME)); // Agent is not allowed
    }

    #[test]
    fn test_coordinator_tools() {
        let tools = get_coordinator_mode_allowed_tools();
        assert!(tools.contains(AGENT_TOOL_NAME));
        assert!(tools.contains(SEND_MESSAGE_TOOL_NAME));
    }
}
