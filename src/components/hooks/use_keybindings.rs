use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub key: String,
    pub modifiers: Vec<String>,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct KeybindingState {
    pub bindings: HashMap<String, Keybinding>,
    pub active_context: Option<String>,
}

impl KeybindingState {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            active_context: None,
        }
    }

    pub fn register_binding(&mut self, key: String, binding: Keybinding) {
        self.bindings.insert(key, binding);
    }

    pub fn get_binding(&self, key: &str) -> Option<&Keybinding> {
        self.bindings.get(key)
    }

    pub fn set_active_context(&mut self, context: Option<String>) {
        self.active_context = context;
    }

    pub fn get_active_context(&self) -> Option<&String> {
        self.active_context.as_ref()
    }
}
