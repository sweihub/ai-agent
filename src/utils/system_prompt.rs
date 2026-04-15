use std::collections::HashMap;

pub fn get_system_prompt(context: &SystemPromptContext) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an AI coding assistant.\n\n");

    prompt.push_str("## Capabilities\n");
    prompt.push_str("- Read, write, and edit files\n");
    prompt.push_str("- Execute commands in a terminal\n");
    prompt.push_str("- Search across codebases\n");
    prompt.push_str("- Use tools to accomplish tasks\n\n");

    prompt.push_str("## Guidelines\n");
    prompt.push_str("- Think step by step\n");
    prompt.push_str("- Verify changes before applying\n");
    prompt.push_str("- Ask for clarification when needed\n\n");

    if let Some(project_type) = &context.project_type {
        prompt.push_str(&format!("## Project Type\n{}\n\n", project_type));
    }

    if let Some(language) = &context.language {
        prompt.push_str(&format!("## Language\n{}\n\n", language));
    }

    prompt
}

#[derive(Debug, Clone)]
pub struct SystemPromptContext {
    pub project_type: Option<String>,
    pub language: Option<String>,
    pub workspace_root: Option<String>,
    pub custom_instructions: Option<String>,
}

impl Default for SystemPromptContext {
    fn default() -> Self {
        Self {
            project_type: None,
            language: None,
            workspace_root: None,
            custom_instructions: None,
        }
    }
}

pub fn get_default_system_prompt() -> String {
    get_system_prompt(&SystemPromptContext::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt() {
        let context = SystemPromptContext {
            language: Some("Rust".to_string()),
            ..Default::default()
        };
        let prompt = get_system_prompt(&context);
        assert!(prompt.contains("Rust"));
    }
}
