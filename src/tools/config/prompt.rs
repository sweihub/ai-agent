// Source: ~/claudecode/openclaudecode/src/tools/ConfigTool/prompt.ts
pub const DESCRIPTION: &str = "Get or set Claude Code configuration settings.";

/// Generate the prompt documentation for the Config tool
pub fn generate_prompt() -> String {
    // In the TypeScript version, this dynamically generates the settings list
    // from SUPPORTED_SETTINGS registry. Since the settings registry lives in
    // the TUI layer (ai-code), we provide the core prompt here.
    r#"Get or set Claude Code configuration settings.

View or change Claude Code settings. Use when the user requests configuration changes, asks about current settings, or when adjusting a setting would benefit them.


## Usage
- **Get current value:** Omit the "value" parameter
- **Set new value:** Include the "value" parameter

## Configurable settings list
The following settings are available for you to change:

### Global Settings (stored in ~/.ai.json)
- theme: "dark", "light", "auto" - UI theme
- editorMode: "default", "vim", "emacs" - Editor keybindings mode
- voiceEnabled: true/false - Enable voice mode

### Project Settings (stored in settings.json)
- model: Model to use for this project
- permissions.defaultMode: "default", "acceptEdits", "plan" - Default permission mode

## Model
- model - Override the default model (sonnet, opus, haiku, best, or full model ID)

## Examples
- Get theme: { "setting": "theme" }
- Set dark theme: { "setting": "theme", "value": "dark" }
- Enable vim mode: { "setting": "editorMode", "value": "vim" }
- Enable verbose: { "setting": "verbose", "value": true }
- Change model: { "setting": "model", "value": "opus" }
- Change permission mode: { "setting": "permissions.defaultMode", "value": "plan" }
"#.to_string()
}
