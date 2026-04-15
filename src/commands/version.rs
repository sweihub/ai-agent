// Source: /data/home/swei/claudecode/openclaudecode/src/commands/version.ts
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCallResult {
    #[serde(rename = "type")]
    pub result_type: String,
    pub value: String,
}

impl CommandCallResult {
    pub fn text(value: impl Into<String>) -> Self {
        Self {
            result_type: "text".to_string(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    #[serde(rename = "argumentHint", skip_serializing_if = "Option::is_none")]
    pub argument_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hidden: Option<bool>,
    #[serde(
        rename = "supportsNonInteractive",
        skip_serializing_if = "Option::is_none"
    )]
    pub supports_non_interactive: Option<bool>,
    #[serde(default)]
    pub command_type: String,
}

impl Command {
    pub fn local(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            argument_hint: None,
            is_hidden: None,
            supports_non_interactive: None,
            command_type: "local".to_string(),
        }
    }

    pub fn prompt(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            argument_hint: None,
            is_hidden: None,
            supports_non_interactive: None,
            command_type: "prompt".to_string(),
        }
    }

    pub fn argument_hint(mut self, hint: impl Into<String>) -> Self {
        self.argument_hint = Some(hint.into());
        self
    }

    pub fn is_hidden(mut self, hidden: bool) -> Self {
        self.is_hidden = Some(hidden);
        self
    }

    pub fn supports_non_interactive(mut self, supported: bool) -> Self {
        self.supports_non_interactive = Some(supported);
        self
    }
}

#[derive(Debug, Clone)]
pub struct CommandContext;

pub fn create_version_command() -> Command {
    Command::local(
        "version",
        "Print the version this session is running (not what autoupdate downloaded)",
    )
    .supports_non_interactive(true)
}
