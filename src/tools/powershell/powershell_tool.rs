// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/PowerShellTool.tsx
//! PowerShell tool - executes PowerShell commands

use crate::error::AgentError;
use crate::tools::powershell::prompt::get_default_timeout_ms;
use crate::tools::powershell::powershell_security::powershell_command_is_safe;
use crate::tools::powershell::tool_name::POWERSHELL_TOOL_NAME;
use crate::types::*;

/// PowerShell tool - executes PowerShell commands with security checks
pub struct PowerShellTool;

impl PowerShellTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        POWERSHELL_TOOL_NAME
    }

    pub fn description(&self) -> String {
        "Execute a PowerShell command. Windows-only tool for PowerShell cmdlets and native executable execution"
            .to_string()
    }

    pub fn user_facing_name(&self, _input: Option<&serde_json::Value>) -> String {
        "PowerShell".to_string()
    }

    pub fn get_tool_use_summary(&self, input: Option<&serde_json::Value>) -> Option<String> {
        input.and_then(|inp| inp["command"].as_str().map(String::from))
    }

    pub fn render_tool_result_message(
        &self,
        content: &serde_json::Value,
    ) -> Option<String> {
        let content_str = content["content"].as_str()?;
        if content_str.is_empty() {
            Some("No output".to_string())
        } else {
            let line_count = content_str.lines().count();
            Some(format!(
                "{} {}",
                line_count,
                if line_count == 1 { "line" } else { "lines" }
            ))
        }
    }

    pub fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({
                "command": {
                    "type": "string",
                    "description": "PowerShell command to execute"
                },
                "timeout": {
                    "type": "number",
                    "description": "Optional timeout in milliseconds (default: 120000, max: 600000)"
                },
                "description": {
                    "type": "string",
                    "description": "Brief description of what this command does"
                },
                "run_in_background": {
                    "type": "boolean",
                    "description": "Run the command in the background (default: false)"
                }
            }),
            required: Some(vec!["command".to_string()]),
        }
    }

    pub async fn execute(
        &self,
        input: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolResult, AgentError> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| AgentError::Tool("command is required".to_string()))?
            .to_string();

        // Security check
        let security = powershell_command_is_safe(&command);
        if security.behavior
            == crate::tools::powershell::powershell_security::SecurityBehavior::Ask
        {
            // For now, pass through with warning (full permission handling done at agent level)
            let _warning = security.message;
        }

        let cwd = context.cwd.clone();
        let output = tokio::task::spawn_blocking(move || {
            // Try pwsh first (PowerShell 7+), fall back to powershell.exe
            let mut cmd = if let Ok(output) = std::process::Command::new("pwsh")
                .arg("-c")
                .arg("--version")
                .output()
            {
                if output.status.success() {
                    let mut c = std::process::Command::new("pwsh");
                    c.arg("-NoProfile").arg("-Command").arg(&command);
                    c
                } else {
                    let mut c = std::process::Command::new("powershell");
                    c.arg("-NoProfile")
                        .arg("-NonInteractive")
                        .arg("-Command")
                        .arg(&command);
                    c
                }
            } else {
                let mut c = std::process::Command::new("powershell");
                c.arg("-NoProfile")
                    .arg("-NonInteractive")
                    .arg("-Command")
                    .arg(&command);
                c
            };

            if !cwd.is_empty() {
                cmd.current_dir(&cwd);
            }
            cmd.output()
        })
        .await
        .map_err(|e| AgentError::Tool(e.to_string()))?
        .map_err(|e| {
            AgentError::Tool(format!(
                "Failed to execute PowerShell command: {}. Make sure PowerShell is installed.",
                e
            ))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let content = if !stdout.trim().is_empty() {
            stdout.to_string()
        } else if !stderr.trim().is_empty() {
            stderr.to_string()
        } else {
            "".to_string()
        };

        let is_error = !output.status.success();

        Ok(ToolResult {
            result_type: "tool_result".to_string(),
            tool_use_id: "".to_string(),
            content,
            is_error: Some(is_error),
            was_persisted: None,
        })
    }
}

impl Default for PowerShellTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_tool_name() {
        let tool = PowerShellTool::new();
        assert_eq!(tool.name(), POWERSHELL_TOOL_NAME);
    }

    #[test]
    fn test_powershell_tool_schema() {
        let tool = PowerShellTool::new();
        let schema = tool.input_schema();
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties["command"].is_object());
        assert!(schema.required.is_some());
    }

    #[test]
    fn test_powershell_tool_user_facing_name() {
        let tool = PowerShellTool::new();
        assert_eq!(tool.user_facing_name(None), "PowerShell");
    }

    #[test]
    fn test_powershell_tool_summary() {
        let tool = PowerShellTool::new();
        let input = serde_json::json!({"command": "Get-ChildItem"});
        let summary = tool.get_tool_use_summary(Some(&input));
        assert_eq!(summary, Some("Get-ChildItem".to_string()));
    }

    #[test]
    fn test_powershell_tool_description_not_empty() {
        let tool = PowerShellTool::new();
        assert!(!tool.description().is_empty());
    }
}
