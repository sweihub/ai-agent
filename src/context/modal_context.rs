use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalContext {
    pub is_open: bool,
    pub modal_type: Option<ModalType>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModalType {
    Confirm,
    Alert,
    Prompt,
    Custom,
}

impl Default for ModalContext {
    fn default() -> Self {
        Self {
            is_open: false,
            modal_type: None,
            data: None,
        }
    }
}

impl ModalContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, modal_type: ModalType) {
        self.is_open = true;
        self.modal_type = Some(modal_type);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.modal_type = None;
        self.data = None;
    }

    pub fn set_data(&mut self, data: serde_json::Value) {
        self.data = Some(data);
    }
}
