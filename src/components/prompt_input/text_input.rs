use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInputProps {
    pub value: String,
    pub placeholder: Option<String>,
    pub multiline: bool,
    pub show_cursor: bool,
    pub columns: usize,
    pub cursor_offset: usize,
    pub on_change: Option<String>,
    pub on_submit: Option<String>,
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: None,
            multiline: false,
            show_cursor: true,
            columns: 80,
            cursor_offset: 0,
            on_change: None,
            on_submit: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextInputState {
    pub value: String,
    pub cursor_offset: usize,
    pub rendered_value: String,
}

impl TextInputState {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor_offset: 0,
            rendered_value: String::new(),
        }
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = value;
        self
    }
}

impl Default for TextInputState {
    fn default() -> Self {
        Self::new()
    }
}
