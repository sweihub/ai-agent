//! Bridge (remote-control) command

use std::env;

pub fn create_bridge_command() -> crate::commands::Command {
    let has_bridge_mode = env::var("AI_CODE_FEATURE_BRIDGE_MODE").is_ok();
    let is_enabled = has_bridge_mode && crate::bridge::is_bridge_enabled();

    crate::commands::Command {
        command_type: crate::commands::CommandType::LocalJsx,
        name: "remote-control".to_string(),
        aliases: vec!["rc".to_string()],
        description: "Connect this terminal for remote-control sessions".to_string(),
        argument_hint: Some("[name]".to_string()),
        is_enabled: Some(is_enabled),
        is_hidden: Some(!is_enabled),
        immediate: true,
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::bridge::Bridge::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
