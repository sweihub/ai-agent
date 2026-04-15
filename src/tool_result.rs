//! Tool result types.

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(tool_name: String, output: String) -> Self {
        Self {
            tool_name,
            success: true,
            output: Some(output),
            error: None,
        }
    }

    pub fn error(tool_name: String, error: String) -> Self {
        Self {
            tool_name,
            success: false,
            output: None,
            error: Some(error),
        }
    }
}
