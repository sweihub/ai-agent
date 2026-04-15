// Source: /data/home/swei/claudecode/openclaudecode/src/utils/editor.ts
use crate::constants::env::system;
use std::path::Path;

pub fn detect_editor() -> Option<String> {
    std::env::var(system::EDITOR)
        .ok()
        .or_else(|| std::env::var(system::VISUAL).ok())
        .or_else(|| {
            if cfg!(target_os = "macos") {
                Some("open".to_string())
            } else if cfg!(target_os = "windows") {
                Some("notepad".to_string())
            } else {
                Some("vim".to_string())
            }
        })
}

pub fn get_editor_args(editor: &str, file_path: &Path) -> Vec<String> {
    match editor {
        "vim" | "vi" | "nano" | "emacs" => vec![file_path.to_string_lossy().to_string()],
        "code" | "code-insiders" => vec![
            "--wait".to_string(),
            file_path.to_string_lossy().to_string(),
        ],
        "subl" | "sublime" => vec![file_path.to_string_lossy().to_string()],
        _ => vec![file_path.to_string_lossy().to_string()],
    }
}

pub fn spawn_editor(file_path: &Path) -> Result<(), String> {
    let editor = detect_editor().ok_or("No editor found")?;
    let args = get_editor_args(&editor, file_path);

    std::process::Command::new(&editor)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to spawn editor: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_editor() {
        let editor = detect_editor();
        assert!(editor.is_some());
    }

    #[test]
    fn test_editor_args() {
        let args = get_editor_args("vim", Path::new("test.txt"));
        assert!(args.contains(&"test.txt".to_string()));
    }
}
