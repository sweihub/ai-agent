// Source: ~/claudecode/openclaudecode/src/tools/TaskListTool/prompt.ts
use crate::utils::agent_swarms::is_agent_swarms_enabled;

pub const DESCRIPTION: &str = "List all tasks in the task list";

pub fn get_prompt() -> String {
    let teammate_use_case = if is_agent_swarms_enabled() {
        "- Before assigning tasks to teammates, to see what's available\n"
    } else {
        ""
    };

    let id_description = if is_agent_swarms_enabled() {
        "- **id**: Task identifier (use with TaskGet, TaskUpdate)"
    } else {
        "- **id**: Task identifier (use with TaskGet, TaskUpdate)"
    };

    let teammate_workflow = if is_agent_swarms_enabled() {
        r#"
## Teammate Workflow

When working as a teammate:
1. After completing your current task, call TaskList to find available work
2. Look for tasks with status 'pending', no owner, and empty blockedBy
3. **Prefer tasks in ID order** (lowest ID first) when multiple tasks are available, as earlier tasks often set up context for later ones
4. Claim an available task using TaskUpdate (set `owner` to your name), or wait for leader assignment
5. If blocked, focus on unblocking tasks or notify the team lead
"#
    } else {
        ""
    };

    format!(
        r#"Use this tool to list all tasks in the task list.

## When to Use This Tool

- To see what tasks are available to work on (status: 'pending', no owner, not blocked)
- To check overall progress on the project
- To find tasks that are blocked and need dependencies resolved
{}- After completing a task, to check for newly unblocked work or claim the next available task
- **Prefer working on tasks in ID order** (lowest ID first) when multiple tasks are available, as earlier tasks often set up context for later ones

## Output

Returns a summary of each task:
{}
- **subject**: Brief description of the task
- **status**: 'pending', 'in_progress', or 'completed'
- **owner**: Agent ID if assigned, empty if available
- **blockedBy**: List of open task IDs that must be resolved first (tasks with blockedBy cannot be claimed until dependencies resolve)

Use TaskGet with a specific task ID to view full details including description and comments.
{}"#,
        teammate_use_case, id_description, teammate_workflow,
    )
}
