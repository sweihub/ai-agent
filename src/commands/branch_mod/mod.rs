//! Branch command

use std::env;

pub fn create_branch_command() -> crate::commands::Command {
    let has_fork_subagent = env::var("AI_CODE_FEATURE_FORK_SUBAGENT").is_ok();
    let aliases = if has_fork_subagent {
        vec![]
    } else {
        vec!["fork".to_string()]
    };

    crate::commands::Command {
        command_type: crate::commands::CommandType::LocalJsx,
        name: "branch".to_string(),
        aliases,
        description: "Create a branch of the current conversation at this point".to_string(),
        argument_hint: Some("[name]".to_string()),
        load: Some(Box::new(|| {
            Box::pin(async {
                Ok(Box::new(crate::commands::branch::Branch::new())
                    as Box<dyn crate::commands::CommandHandler>)
            })
        })),
    }
}
