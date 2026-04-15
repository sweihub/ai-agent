pub fn get_file_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
}

pub fn get_file_name(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

pub fn get_file_stem(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
}

pub fn get_parent_directory(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
}

pub fn join_paths(base: &str, child: &str) -> String {
    std::path::Path::new(base)
        .join(child)
        .to_string_lossy()
        .to_string()
}

pub fn is_absolute_path(path: &str) -> bool {
    std::path::Path::new(path).is_absolute()
}

pub fn normalize_path(path: &str) -> String {
    std::path::Path::new(path)
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string())
}
