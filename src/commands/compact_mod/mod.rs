//! Compact command

use crate::constants::env::system;

pub fn create_compact_command() -> crate::commands::Command {
    let is_disabled = std::env::var(system::DISABLE_COMPACT)
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    crate::commands::Command {
        command_type: crate::commands::CommandType::Local,
        name: "compact".to_string(),
        description: "Clear conversation history but keep a summary in context".to_string(),
        is_enabled: Some(!is_disabled),
        supports_non_interactive: true,
        argument_hint: Some("<optional custom summarization instructions>".to_string()),
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::compact::Compact::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
