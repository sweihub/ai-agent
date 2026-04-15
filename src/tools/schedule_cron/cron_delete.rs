// Source: ~/claudecode/openclaudecode/src/tools/ScheduleCronTool/CronDeleteTool.ts
use crate::tools::schedule_cron::prompt::{
    build_cron_delete_prompt, cron_delete_description, cron_delete_tool_name,
    is_durable_cron_enabled, is_kairos_cron_enabled,
};
use crate::types::*;
use crate::utils::cron::{get_cron_file_path, list_all_cron_tasks, remove_cron_tasks};
use crate::utils::teammate::get_teammate_context;

pub struct CronDeleteTool;

impl CronDeleteTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        cron_delete_tool_name()
    }

    pub fn search_hint(&self) -> &str {
        "cancel a scheduled cron job"
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

    pub fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({
                "id": {
                    "type": "string",
                    "description": "Job ID returned by CronCreate."
                }
            }),
            required: Some(vec!["id".to_string()]),
        }
    }

    pub fn to_auto_classifier_input(&self, input: &serde_json::Value) -> String {
        input["id"].as_str().unwrap_or("").to_string()
    }

    pub async fn description(&self) -> String {
        cron_delete_description()
    }

    pub async fn prompt(&self) -> String {
        build_cron_delete_prompt(is_durable_cron_enabled())
    }

    pub fn get_path(&self) -> Option<String> {
        get_cron_file_path()
    }

    pub async fn validate_input(
        &self,
        input: &serde_json::Value,
    ) -> ValidationResult {
        let id = match input["id"].as_str() {
            Some(id) => id.to_string(),
            None => {
                return ValidationResult {
                    result: false,
                    message: "id is required".to_string(),
                    error_code: Some(0),
                }
            }
        };

        let tasks = list_all_cron_tasks().await;
        let task = tasks.iter().find(|t| t.id == id);

        if task.is_none() {
            return ValidationResult {
                result: false,
                message: format!("No scheduled job with id '{}'", id),
                error_code: Some(1),
            };
        }

        // Teammates may only delete their own crons.
        let ctx = get_teammate_context();
        if let Some(teammate_ctx) = ctx {
            if let Some(task) = task {
                if task.agent_id != teammate_ctx.agent_id {
                    return ValidationResult {
                        result: false,
                        message: format!(
                            "Cannot delete cron job '{}': owned by another agent",
                            id
                        ),
                        error_code: Some(2),
                    };
                }
            }
        }

        ValidationResult {
            result: true,
            message: String::new(),
            error_code: None,
        }
    }

    pub async fn execute(
        &self,
        input: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, crate::error::AgentError> {
        let id = input["id"]
            .as_str()
            .ok_or_else(|| crate::error::AgentError::Tool("id is required".to_string()))?
            .to_string();

        remove_cron_tasks(&[id.clone()]).await;

        let content = format!("Cancelled job {}.", id);

        Ok(ToolResult {
            result_type: "text".to_string(),
            tool_use_id: "".to_string(),
            content,
            is_error: None,
            data: Some(serde_json::json!({ "id": id })),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_delete_tool_name() {
        let tool = CronDeleteTool::new();
        assert!(!tool.name().is_empty());
    }

    #[test]
    fn test_cron_delete_tool_input_schema_has_id() {
        let tool = CronDeleteTool::new();
        let schema = tool.input_schema();
        assert!(schema.properties.get("id").is_some());
    }
}
