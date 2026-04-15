use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddDirectoryResultType {
    Success,
    EmptyPath,
    PathNotFound,
    NotADirectory,
    AlreadyInWorkingDirectory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "resultType")]
pub enum AddDirectoryResult {
    #[serde(rename = "success")]
    Success { absolute_path: String },
    #[serde(rename = "emptyPath")]
    EmptyPath,
    #[serde(rename = "pathNotFound")]
    PathNotFound {
        directory_path: String,
        absolute_path: String,
    },
    #[serde(rename = "notADirectory")]
    NotADirectory {
        directory_path: String,
        absolute_path: String,
    },
    #[serde(rename = "alreadyInWorkingDirectory")]
    AlreadyInWorkingDirectory {
        directory_path: String,
        working_dir: String,
    },
}

pub fn validate_directory_for_workspace(
    directory_path: &str,
    _permission_context: &impl ToolPermissionContext,
) -> AddDirectoryResult {
    if directory_path.is_empty() {
        return AddDirectoryResult::EmptyPath;
    }
    AddDirectoryResult::Success {
        absolute_path: directory_path.to_string(),
    }
}

pub trait ToolPermissionContext {}

pub fn add_dir_help_message(result: &AddDirectoryResult) -> String {
    match result {
        AddDirectoryResult::EmptyPath => "Please provide a directory path.".to_string(),
        AddDirectoryResult::Success { absolute_path } => {
            format!("Added {} as a working directory.", absolute_path)
        }
        _ => "Invalid directory".to_string(),
    }
}
