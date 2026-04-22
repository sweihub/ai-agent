// Source: /data/home/swei/claudecode/openclaudecode/src/tools/PowerShellTool/toolName.ts
//! PowerShell tool name constant

/// PowerShell tool name
pub const POWERSHELL_TOOL_NAME: &str = "PowerShell";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powershell_tool_name() {
        assert_eq!(POWERSHELL_TOOL_NAME, "PowerShell");
    }

    #[test]
    fn test_powershell_tool_name_not_empty() {
        assert!(!POWERSHELL_TOOL_NAME.is_empty());
    }
}
