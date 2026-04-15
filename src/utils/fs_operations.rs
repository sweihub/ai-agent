use std::path::Path;

pub fn read_file_sync(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

pub fn write_file_sync(path: &Path, content: &str) -> Result<(), String> {
    std::fs::write(path, content).map_err(|e| e.to_string())
}

pub fn append_file_sync(path: &Path, content: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())
}

pub fn create_dir_sync(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())
}

pub fn remove_file_sync(path: &Path) -> Result<(), String> {
    std::fs::remove_file(path).map_err(|e| e.to_string())
}

pub fn remove_dir_sync(path: &Path) -> Result<(), String> {
    std::fs::remove_dir_all(path).map_err(|e| e.to_string())
}

pub fn exists(path: &Path) -> bool {
    path.exists()
}

pub fn is_dir(path: &Path) -> bool {
    path.is_dir()
}

pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

pub fn list_dir(path: &Path) -> Result<Vec<String>, String> {
    std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .map(|entry| {
            entry
                .map(|e| e.path().to_string_lossy().to_string())
                .map_err(|e| e.to_string())
        })
        .collect()
}
