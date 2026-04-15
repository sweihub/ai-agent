//! Agents command

pub fn create_agents_command() -> crate::commands::Command {
    crate::commands::Command {
        command_type: crate::commands::CommandType::LocalJsx,
        name: "agents".to_string(),
        description: "Manage agent configurations".to_string(),
        argument_hint: None,
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::agents::Agents::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
