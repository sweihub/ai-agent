use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOverlayContext {
    pub is_active: bool,
    pub prompt_text: Option<String>,
    pub suggestions: Vec<String>,
}

impl Default for PromptOverlayContext {
    fn default() -> Self {
        Self {
            is_active: false,
            prompt_text: None,
            suggestions: Vec::new(),
        }
    }
}

impl PromptOverlayContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn activate(&mut self, prompt: &str) {
        self.is_active = true;
        self.prompt_text = Some(prompt.to_string());
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.prompt_text = None;
        self.suggestions.clear();
    }

    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }
}
