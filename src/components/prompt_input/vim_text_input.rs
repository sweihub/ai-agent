use super::text_input::{TextInputProps, TextInputState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimTextInputProps {
    #[serde(flatten)]
    pub base: TextInputProps,
    pub initial_mode: Option<VimMode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "uppercase")]
pub enum VimMode {
    Insert,
    Normal,
}

impl Default for VimTextInputProps {
    fn default() -> Self {
        Self {
            base: TextInputProps::default(),
            initial_mode: Some(VimMode::Insert),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimInputState {
    #[serde(flatten)]
    pub base: TextInputState,
    pub mode: VimMode,
}

impl VimInputState {
    pub fn new() -> Self {
        Self {
            base: TextInputState::new(),
            mode: VimMode::Insert,
        }
    }

    pub fn switch_mode(&mut self, mode: VimMode) {
        self.mode = mode;
    }

    pub fn is_insert_mode(&self) -> bool {
        self.mode == VimMode::Insert
    }

    pub fn is_normal_mode(&self) -> bool {
        self.mode == VimMode::Normal
    }
}

impl Default for VimInputState {
    fn default() -> Self {
        Self::new()
    }
}
