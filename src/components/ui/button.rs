use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonProps {
    pub label: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    pub on_click: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            disabled: false,
            on_click: None,
        }
    }
}

impl ButtonProps {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            ..Default::default()
        }
    }

    pub fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn with_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}
