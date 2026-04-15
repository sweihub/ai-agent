use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpinnerProps {
    pub size: SpinnerSize,
    pub color: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SpinnerSize {
    Small,
    Medium,
    Large,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        Self {
            size: SpinnerSize::Medium,
            color: None,
            label: None,
        }
    }
}

impl SpinnerProps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }
}
