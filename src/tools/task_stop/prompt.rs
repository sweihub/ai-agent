// Source: ~/claudecode/openclaudecode/src/tools/TaskStopTool/prompt.ts
pub const TASK_STOP_TOOL_NAME: &str = "TaskStop";

pub const DESCRIPTION: &str = r#"
- Stops a running background task by its ID
- Takes a task_id parameter identifying the task to stop
- Returns a success or failure status
- Use this tool when you need to terminate a long-running task
"#;
