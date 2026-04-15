//! Shell tool utility types and constants.

/// Supported shell types
pub const SHELL_TYPES: [&str; 2] = ["bash", "powershell"];

/// Shell type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    PowerShell,
}

impl ShellType {
    /// Create ShellType from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "powershell" | "pwsh" => Some(Self::PowerShell),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::PowerShell => "powershell",
        }
    }
}

impl Default for ShellType {
    fn default() -> Self {
        Self::Bash
    }
}

/// Default shell for hooks
pub const DEFAULT_HOOK_SHELL: ShellType = ShellType::Bash;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_from_str() {
        assert_eq!(ShellType::from_str("bash"), Some(ShellType::Bash));
        assert_eq!(ShellType::from_str("BASH"), Some(ShellType::Bash));
        assert_eq!(
            ShellType::from_str("powershell"),
            Some(ShellType::PowerShell)
        );
        assert_eq!(ShellType::from_str("pwsh"), Some(ShellType::PowerShell));
        assert_eq!(ShellType::from_str("unknown"), None);
    }

    #[test]
    fn test_shell_type_as_str() {
        assert_eq!(ShellType::Bash.as_str(), "bash");
        assert_eq!(ShellType::PowerShell.as_str(), "powershell");
    }
}
