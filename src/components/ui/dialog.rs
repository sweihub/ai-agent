use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogProps {
    pub title: String,
    pub message: Option<String>,
    pub is_open: bool,
    pub dialog_type: DialogType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DialogType {
    Alert,
    Confirm,
    Prompt,
}

impl Default for DialogProps {
    fn default() -> Self {
        Self {
            title: String::new(),
            message: None,
            is_open: false,
            dialog_type: DialogType::Alert,
        }
    }
}

impl DialogProps {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            ..Default::default()
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn with_type(mut self, dialog_type: DialogType) -> Self {
        self.dialog_type = dialog_type;
        self
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}
