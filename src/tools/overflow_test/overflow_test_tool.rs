// Source: ~/claudecode/openclaudecode/src/tools/OverflowTestTool/OverflowTestTool.ts
//! Overflow test tool - placeholder for unimplemented functionality

use crate::types::*;

use super::constants::OVERFLOW_TEST_TOOL_NAME;

/// Overflow test tool - placeholder for testing overflow behavior
/// TypeScript exports null (feature-gated/not implemented)
pub struct OverflowTestTool;

impl OverflowTestTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        OVERFLOW_TEST_TOOL_NAME
    }

    pub fn description(&self) -> &str {
        "Test overflow behavior (not implemented)"
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
        Err(crate::error::AgentError::ToolNotImplemented(
            "Overflow test tool is not implemented".to_string(),
        ))
    }
}

impl Default for OverflowTestTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overflow_test_tool_name() {
        let tool = OverflowTestTool::new();
        assert_eq!(tool.name(), OVERFLOW_TEST_TOOL_NAME);
    }

    #[test]
    fn test_overflow_test_tool_schema() {
        let tool = OverflowTestTool::new();
        let schema = tool.input_schema();
        assert_eq!(schema.schema_type, "object");
    }
}
