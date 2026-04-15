use std::path::Path;

pub fn detect_language(filename: &str) -> Option<String> {
    let ext = Path::new(filename).extension().and_then(|e| e.to_str())?;

    match ext.to_lowercase().as_str() {
        "rs" => Some("rust".to_string()),
        "js" => Some("javascript".to_string()),
        "ts" => Some("typescript".to_string()),
        "tsx" => Some("typescript".to_string()),
        "jsx" => Some("javascript".to_string()),
        "py" => Some("python".to_string()),
        "go" => Some("go".to_string()),
        "java" => Some("java".to_string()),
        "c" => Some("c".to_string()),
        "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
        "h" => Some("c".to_string()),
        "hpp" => Some("cpp".to_string()),
        "rb" => Some("ruby".to_string()),
        "php" => Some("php".to_string()),
        "swift" => Some("swift".to_string()),
        "kt" | "kts" => Some("kotlin".to_string()),
        "scala" => Some("scala".to_string()),
        "cs" => Some("csharp".to_string()),
        "fs" => Some("fsharp".to_string()),
        "html" => Some("html".to_string()),
        "css" => Some("css".to_string()),
        "scss" | "sass" => Some("scss".to_string()),
        "json" => Some("json".to_string()),
        "yaml" | "yml" => Some("yaml".to_string()),
        "xml" => Some("xml".to_string()),
        "md" | "markdown" => Some("markdown".to_string()),
        "sql" => Some("sql".to_string()),
        "sh" | "bash" => Some("bash".to_string()),
        "zsh" => Some("zsh".to_string()),
        "ps1" => Some("powershell".to_string()),
        "dockerfile" => Some("dockerfile".to_string()),
        _ => None,
    }
}

pub fn get_language_for_code(code: &str) -> Option<String> {
    if code.contains("fn ") && code.contains("->") && code.contains("let ") {
        return Some("rust".to_string());
    }
    if code.contains("function ") || code.contains("const ") || code.contains("let ") {
        if code.contains(": string") || code.contains(": number") {
            return Some("typescript".to_string());
        }
        return Some("javascript".to_string());
    }
    if code.contains("def ") && code.contains(":") {
        return Some("python".to_string());
    }
    if code.contains("func ") && code.contains("package ") {
        return Some("go".to_string());
    }
    None
}
