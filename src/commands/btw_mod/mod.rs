//! BTW command - ask a quick side question

pub fn create_btw_command() -> crate::commands::Command {
    crate::commands::Command {
        command_type: crate::commands::CommandType::LocalJsx,
        name: "btw".to_string(),
        description: "Ask a quick side question without interrupting the main conversation"
            .to_string(),
        argument_hint: Some("<question>".to_string()),
        immediate: true,
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::btw::Btw::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
