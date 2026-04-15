// Source: /data/home/swei/claudecode/openclaudecode/src/commands/add-dir/validation.ts
//! Add directory validation

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AddDirectoryResult {
    Success {
        absolute_path: PathBuf,
    },
    EmptyPath,
    PathNotFound {
        directory_path: String,
        absolute_path: PathBuf,
    },
    NotADirectory {
        directory_path: String,
        absolute_path: PathBuf,
    },
    AlreadyInWorkingDirectory {
        directory_path: String,
        working_dir: String,
    },
}

pub async fn validate_directory_for_workspace(
    directory_path: &str,
    _permission_context: &crate::types::PermissionContext,
) -> AddDirectoryResult {
    if directory_path.is_empty() {
        return AddDirectoryResult::EmptyPath;
    }

    let absolute_path = PathBuf::from(directory_path);

    match std::fs::metadata(&absolute_path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return AddDirectoryResult::NotADirectory {
                    directory_path: directory_path.to_string(),
                    absolute_path,
                };
            }
        }
        Err(e) => {
            let code = e.kind();
            if code == std::io::ErrorKind::NotFound
                || code == std::io::ErrorKind::PermissionDenied
                || code == std::io::ErrorKind::NotADirectory
            {
                return AddDirectoryResult::PathNotFound {
                    directory_path: directory_path.to_string(),
                    absolute_path,
                };
            }
            return AddDirectoryResult::PathNotFound {
                directory_path: directory_path.to_string(),
                absolute_path,
            };
        }
    }

    AddDirectoryResult::Success { absolute_path }
}

pub fn add_dir_help_message(result: &AddDirectoryResult) -> String {
    match result {
        AddDirectoryResult::EmptyPath => "Please provide a directory path.".to_string(),
        AddDirectoryResult::PathNotFound { absolute_path, .. } => {
            format!("Path {} was not found.", absolute_path.display())
        }
        AddDirectoryResult::NotADirectory { directory_path, .. } => {
            format!("{} is not a directory.", directory_path)
        }
        AddDirectoryResult::AlreadyInWorkingDirectory {
            directory_path,
            working_dir,
        } => {
            format!(
                "{} is already accessible within the existing working directory {}.",
                directory_path, working_dir
            )
        }
        AddDirectoryResult::Success { absolute_path } => {
            format!("Added {} as a working directory.", absolute_path.display())
        }
    }
}
