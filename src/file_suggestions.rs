#![allow(dead_code)]

pub async fn fetch_file_suggestions(
    _path: &str,
) -> Result<Vec<FileSuggestion>, Box<dyn std::error::Error>> {
    Ok(vec![])
}

#[derive(Debug, Clone)]
pub struct FileSuggestion {
    pub path: String,
    pub score: f32,
}
