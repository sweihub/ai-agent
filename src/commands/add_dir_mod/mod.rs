//! Add-dir command

use crate::commands::Command;

pub fn create_add_dir_command() -> Command {
    Command {
        command_type: crate::commands::CommandType::LocalJsx,
        name: "add-dir".to_string(),
        description: "Add a new working directory".to_string(),
        argument_hint: Some("<path>".to_string()),
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::add_dir::AddDir::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
