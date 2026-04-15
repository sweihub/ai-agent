use std::path::{Path, PathBuf};

pub fn normalize_path(path: &str) -> String {
    let mut result = path.replace('\\', "/");
    while result.contains("//") {
        result = result.replace("//", "/");
    }
    result
}

pub fn get_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

pub fn get_filename(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

pub fn get_directory(path: &str) -> Option<String> {
    Path::new(path)
        .parent()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string())
}

pub fn join_paths(a: &str, b: &str) -> String {
    Path::new(a).join(b).to_string_lossy().to_string()
}

pub fn is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute()
}

pub fn make_absolute(path: &str, base: &str) -> String {
    if is_absolute(path) {
        return path.to_string();
    }
    join_paths(base, path)
}

pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}
