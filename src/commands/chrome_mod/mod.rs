//! Chrome command

pub fn create_chrome_command() -> crate::commands::Command {
    let is_non_interactive = crate::bootstrap::is_non_interactive_session();

    crate::commands::Command {
        command_type: crate::commands::CommandType::LocalJsx,
        name: "chrome".to_string(),
        description: "Claude in Chrome (Beta) settings".to_string(),
        availability: vec!["claude-ai".to_string()],
        is_enabled: Some(!is_non_interactive),
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::chrome::Chrome::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
