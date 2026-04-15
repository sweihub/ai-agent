use crate::components::messages::{AssistantTextMessage, UserTextMessage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplScreen {
    pub messages: Vec<ReplMessage>,
    pub input_value: String,
    pub is_processing: bool,
    pub cursor_position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ReplMessage {
    User(UserTextMessage),
    Assistant(AssistantTextMessage),
    System(String),
}

impl Default for ReplScreen {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input_value: String::new(),
            is_processing: false,
            cursor_position: 0,
        }
    }
}

impl ReplScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_message(&mut self, message: ReplMessage) {
        self.messages.push(message);
    }

    pub fn set_input(&mut self, value: String) {
        self.input_value = value;
        self.cursor_position = value.len();
    }

    pub fn clear_input(&mut self) {
        self.input_value.clear();
        self.cursor_position = 0;
    }

    pub fn set_processing(&mut self, processing: bool) {
        self.is_processing = processing;
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
}
