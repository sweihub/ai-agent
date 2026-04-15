//! Clear command

pub fn create_clear_command() -> crate::commands::Command {
    crate::commands::Command {
        command_type: crate::commands::CommandType::Local,
        name: "clear".to_string(),
        description: "Clear conversation history and free up context".to_string(),
        aliases: vec!["reset".to_string(), "new".to_string()],
        supports_non_interactive: false,
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::clear::Clear::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
