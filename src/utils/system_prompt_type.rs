// Source: ~/claudecode/openclaudecode/src/utils/systemPromptType.rs

/// Branded type for system prompt arrays.
///
/// This module is intentionally dependency-free so it can be imported
/// from anywhere without risking circular initialization issues.
#[derive(Debug, Clone)]
pub struct SystemPrompt(Vec<String>);

impl SystemPrompt {
    /// Create a new system prompt from a vector of strings.
    pub fn new(prompts: Vec<String>) -> Self {
        Self(prompts)
    }

    /// Get the underlying prompts as a slice.
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    /// Get the underlying prompts as a vec.
    pub fn into_vec(self) -> Vec<String> {
        self.0
    }
}

impl AsRef<[String]> for SystemPrompt {
    fn as_ref(&self) -> &[String] {
        &self.0
    }
}

impl From<Vec<String>> for SystemPrompt {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for SystemPrompt {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Convert a vector of strings to a SystemPrompt.
pub fn as_system_prompt(value: Vec<String>) -> SystemPrompt {
    SystemPrompt::new(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt() {
        let prompts = vec!["prompt1".to_string(), "prompt2".to_string()];
        let sp = as_system_prompt(prompts);
        assert_eq!(sp.len(), 2);
        assert_eq!(sp[0], "prompt1");
    }
}
