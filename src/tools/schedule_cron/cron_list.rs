// Source: ~/claudecode/openclaudecode/src/tools/ScheduleCronTool/CronListTool.ts
use crate::tools::schedule_cron::prompt::{
    build_cron_list_prompt, cron_list_description, cron_list_tool_name, is_durable_cron_enabled,
    is_kairos_cron_enabled,
};
use crate::types::*;
use crate::utils::cron::{cron_to_human, list_all_cron_tasks};
use crate::utils::format::truncate;
use crate::utils::teammate::get_teammate_context;

pub struct CronListTool;

impl CronListTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        cron_list_tool_name()
    }

    pub fn search_hint(&self) -> &str {
        "list active cron jobs"
    }

    pub fn max_result_size_chars(&self) -> usize {
        100_000
    }

    pub fn should_defer(&self) -> bool {
        true
    }

    pub fn is_enabled(&self) -> bool {
        is_kairos_cron_enabled()
    }

    pub fn is_concurrency_safe(&self) -> bool {
        true
    }

    pub fn is_read_only(&self) -> bool {
        true
    }

    pub async fn description(&self) -> String {
        cron_list_description()
    }

    pub async fn prompt(&self) -> String {
        build_cron_list_prompt(is_durable_cron_enabled())
    }

    pub fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({}),
            required: None,
        }
    }

    pub async fn execute(
        &self,
        _input: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, crate::error::AgentError> {
        let all_tasks = list_all_cron_tasks().await;

        // Teammates only see their own crons; team lead (no ctx) sees all.
        let ctx = get_teammate_context();
        let tasks = if let Some(teammate_ctx) = ctx {
            all_tasks
                .into_iter()
                .filter(|t| t.agent_id == teammate_ctx.agent_id)
                .collect::<Vec<_>>()
        } else {
            all_tasks
        };

        let jobs: Vec<serde_json::Value> = tasks
            .iter()
            .map(|t| {
                let mut job = serde_json::json!({
                    "id": t.id,
                    "cron": t.cron,
                    "human_schedule": cron_to_human(&t.cron),
                    "prompt": &t.prompt,
                });
                if t.recurring {
                    if let Some(obj) = job.as_object_mut() {
                        obj.insert("recurring".to_string(), serde_json::json!(true));
                    }
                }
                if t.durable == Some(false) {
                    if let Some(obj) = job.as_object_mut() {
                        obj.insert("durable".to_string(), serde_json::json!(false));
                    }
                }
                job
            })
            .collect();

        let output = serde_json::json!({ "jobs": jobs });

        let content = if jobs.is_empty() {
            "No scheduled jobs.".to_string()
        } else {
            jobs.iter()
                .map(|j| {
                    let id = j["id"].as_str().unwrap_or("");
                    let human_schedule = j["human_schedule"].as_str().unwrap_or("");
                    let recurring = j.get("recurring").and_then(|v| v.as_bool()).unwrap_or(false);
                    let durable = j.get("durable").and_then(|v| v.as_bool());
                    let prompt = j["prompt"].as_str().unwrap_or("");

                    let recurring_str = if recurring { " (recurring)" } else { " (one-shot)" };
                    let durable_str = if durable == Some(false) {
                        " [session-only]"
                    } else {
                        ""
                    };
                    format!(
                        "{} — {}{}{}: {}",
                        id,
                        human_schedule,
                        recurring_str,
                        durable_str,
                        truncate(prompt, 80, true)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(ToolResult {
            result_type: "text".to_string(),
            tool_use_id: "".to_string(),
            content,
            is_error: None,
            data: Some(output),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_list_tool_name() {
        let tool = CronListTool::new();
        assert!(!tool.name().is_empty());
    }

    #[test]
    fn test_cron_list_tool_is_read_only() {
        let tool = CronListTool::new();
        assert!(tool.is_read_only());
    }

    #[test]
    fn test_cron_list_tool_is_concurrency_safe() {
        let tool = CronListTool::new();
        assert!(tool.is_concurrency_safe());
    }
}
