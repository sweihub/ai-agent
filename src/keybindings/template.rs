// Source: /data/home/swei/claudecode/openclaudecode/src/keybindings/template.ts
//! Keybinding template

pub fn get_keybinding_template() -> String {
    r#"[
  {
    "context": "Chat",
    "bindings": {
      "enter": "submit",
      "shift+enter": "newline",
      "ctrl+k": "command:clear"
    }
  },
  {
    "context": "Global",
    "bindings": {
      "ctrl+o": "app:toggleTranscript",
      "ctrl+/": "app:toggleHelp"
    }
  }
]"#
    .to_string()
}
