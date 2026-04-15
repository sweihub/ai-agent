//! Default keybindings

use std::collections::HashMap;

pub fn get_default_bindings() -> HashMap<String, HashMap<String, String>> {
    let mut bindings = HashMap::new();

    let mut chat_bindings = HashMap::new();
    chat_bindings.insert("enter".to_string(), "submit".to_string());
    chat_bindings.insert("shift+enter".to_string(), "newline".to_string());
    bindings.insert("Chat".to_string(), chat_bindings);

    let mut global_bindings = HashMap::new();
    global_bindings.insert("ctrl+o".to_string(), "app:toggleTranscript".to_string());
    global_bindings.insert("ctrl+/".to_string(), "app:toggleHelp".to_string());
    bindings.insert("Global".to_string(), global_bindings);

    bindings
}
