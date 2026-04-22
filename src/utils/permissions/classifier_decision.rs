// Source: ~/claudecode/openclaudecode/src/utils/permissions/classifierDecision.ts
#![allow(dead_code)]

//! Tools that are safe and don't need any classifier checking.
//! Used by the auto mode classifier to skip unnecessary API calls.

use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    /// Tools that are safe and don't need any classifier checking.
    /// Does NOT include write/edit tools — those are handled by the
    /// accept_edits fast path (allowed in cwd, classified outside cwd).
    static ref SAFE_YOLO_ALLOWLISTED_TOOLS: HashSet<&'static str> = {
        let mut set = HashSet::new();

        // Read-only file operations
        set.insert("FileRead");

        // Search / read-only
        set.insert("Grep");
        set.insert("Glob");
        set.insert("LSP");
        set.insert("ToolSearch");
        set.insert("ListMcpResources");
        set.insert("ReadMcpResource");

        // Task management (metadata only)
        set.insert("TodoWrite");
        set.insert("TaskCreate");
        set.insert("TaskGet");
        set.insert("TaskUpdate");
        set.insert("TaskList");
        set.insert("TaskStop");
        set.insert("TaskOutput");

        // Plan mode / UI
        set.insert("AskUserQuestion");
        set.insert("EnterPlanMode");
        set.insert("ExitPlanMode");

        // Swarm coordination
        set.insert("TeamCreate");
        set.insert("TeamDelete");
        set.insert("SendMessage");

        // Workflow orchestration — subagents go through can_use_tool individually
        // set.insert("Workflow"); // conditionally added

        // Misc safe
        set.insert("Sleep");

        // Ant-only safe tools (conditionally added based on feature flags)
        // set.insert("TerminalCapture");
        // set.insert("OverflowTest");
        // set.insert("VerifyPlanExecution");

        // Internal classifier tool
        set.insert("classify_result");

        set
    };
}

/// Returns whether a tool name is on the auto mode allowlist.
pub fn is_auto_mode_allowlisted_tool(tool_name: &str) -> bool {
    SAFE_YOLO_ALLOWLISTED_TOOLS.contains(tool_name)
}
