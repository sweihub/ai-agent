// Source: /data/home/swei/claudecode/openclaudecode/src/commands/keybindings/keybindings.ts
#![allow(dead_code)]

use std::collections::HashMap;

pub struct KeyBinding {
    pub key: String,
    pub action: String,
    pub description: String,
}

pub fn get_default_keybindings() -> Vec<KeyBinding> {
    vec![KeyBinding {
        key: "Ctrl+C".to_string(),
        action: "copy".to_string(),
        description: "Copy selection".to_string(),
    }]
}
