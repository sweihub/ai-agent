//! Agents-platform command

pub fn create_agents_platform_command() -> crate::commands::Command {
    crate::commands::Command {
        command_type: crate::commands::CommandType::Local,
        name: "agents-platform".to_string(),
        description: "Unavailable in restored development build.".to_string(),
        argument_hint: None,
        supports_non_interactive: true,
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::agents_platform::AgentsPlatform)
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
