// Source: /data/home/swei/claudecode/openclaudecode/src/keybindings/schema.ts
//! Keybinding schema

pub fn get_keybinding_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "items": {
            "type": "object",
            "required": ["context", "bindings"],
            "properties": {
                "context": {
                    "type": "string",
                    "enum": ["Global", "Chat", "Autocomplete", "Confirmation", "Help", "Transcript"]
                },
                "bindings": {
                    "type": "object",
                    "additionalProperties": {
                        "type": ["string", "null"]
                    }
                }
            }
        }
    })
}
