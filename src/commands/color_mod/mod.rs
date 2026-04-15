//! Color command

pub fn create_color_command() -> crate::commands::Command {
    crate::commands::Command {
        command_type: crate::commands::CommandType::LocalJsx,
        name: "color".to_string(),
        description: "Set the prompt bar color for this session".to_string(),
        argument_hint: Some("<color|default>".to_string()),
        immediate: true,
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::color::Color::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
