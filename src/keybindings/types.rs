// Source: /data/home/swei/claudecode/openclaudecode/src/utils/filePersistence/types.ts
//! Keybinding types

use std::collections::HashMap;

pub type KeybindingContextName = String;

#[derive(Debug, Clone)]
pub struct KeybindingBlock {
    pub context: String,
    pub bindings: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ParsedBinding {
    pub chord: Vec<crate::keybindings::parser::ParsedKeystroke>,
    pub action: Option<String>,
    pub context: String,
}
