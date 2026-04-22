//! Memory directory scanning primitives.

use std::fs;
use std::path::Path;

use crate::memdir::memory_types::{MemoryType, parse_frontmatter};

/// Maximum number of memory files to scan
pub const MAX_MEMORY_FILES: usize = 200;
/// Maximum lines to read for frontmatter parsing
const FRONTMATTER_MAX_LINES: usize = 30;

/// Header information for a memory file
#[derive(Debug, Clone)]
pub struct MemoryHeader {
    pub filename: String,
    pub file_path: String,
    pub mtime_ms: u64,
    pub description: Option<String>,
    pub memory_type: Option<MemoryType>,
}

/// Scan a memory directory for .md files, read their frontmatter,
/// and return a header list sorted newest-first (capped at MAX_MEMORY_FILES).
pub async fn scan_memory_files(memory_dir: &str) -> Vec<MemoryHeader> {
    let path = Path::new(memory_dir);

    if !path.exists() {
        return Vec::new();
    }

    let mut md_files = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() {
                if let Some(ext) = file_path.extension() {
                    if ext == "md" {
                        if let Some(name) = file_path.file_name() {
                            let name_str = name.to_string_lossy();
                            // Exclude MEMORY.md (already loaded in system prompt)
                            if name_str != "MEMORY.md" {
                                md_files.push(file_path);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut headers = Vec::new();

    for file_path in md_files {
        if let Some(filename) = file_path.file_name() {
            let filename_str = filename.to_string_lossy().to_string();
            let file_path_str = file_path.to_string_lossy().to_string();

            // Get modification time
            let mtime_ms = file_path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            // Read frontmatter
            let (description, memory_type) = if let Ok(content) = read_frontmatter_lines(&file_path)
            {
                if let Some(fm) = parse_frontmatter(&content) {
                    (Some(fm.description), Some(fm.memory_type))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            headers.push(MemoryHeader {
                filename: filename_str,
                file_path: file_path_str,
                mtime_ms,
                description,
                memory_type,
            });
        }
    }

    // Sort by mtime newest-first
    headers.sort_by(|a, b| b.mtime_ms.cmp(&a.mtime_ms));

    // Cap at MAX_MEMORY_FILES
    headers.truncate(MAX_MEMORY_FILES);

    headers
}

/// Read only the first N lines of a file (for frontmatter parsing)
fn read_frontmatter_lines(path: &Path) -> std::io::Result<String> {
    use std::io::{BufRead, BufReader};

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        if i >= FRONTMATTER_MAX_LINES {
            break;
        }
        if let Ok(l) = line {
            lines.push(l);
        }
    }

    Ok(lines.join("\n"))
}

/// Format memory headers as a text manifest:
/// one line per file with [type] filename (timestamp): description
pub fn format_memory_manifest(memories: &[MemoryHeader]) -> String {
    memories
        .iter()
        .map(|m| {
            let tag = m
                .memory_type
                .as_ref()
                .map(|t| format!("[{}] ", t.as_str()))
                .unwrap_or_default();

            let ts = chrono::DateTime::from_timestamp_millis(m.mtime_ms as i64)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "unknown".to_string());

            match &m.description {
                Some(desc) => format!("- {}{} ({}): {}", tag, m.filename, ts, desc),
                None => format!("- {}{} ({})", tag, m.filename, ts),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_format_memory_manifest() {
        let headers = vec![
            MemoryHeader {
                filename: "user_test.md".to_string(),
                file_path: "/tmp/memory/user_test.md".to_string(),
                mtime_ms: 1700000000000,
                description: Some("Test description".to_string()),
                memory_type: Some(MemoryType::User),
            },
            MemoryHeader {
                filename: "feedback_test.md".to_string(),
                file_path: "/tmp/memory/feedback_test.md".to_string(),
                mtime_ms: 1700000000000,
                description: None,
                memory_type: Some(MemoryType::Feedback),
            },
        ];

        let manifest = format_memory_manifest(&headers);
        assert!(manifest.contains("user_test.md"));
        assert!(manifest.contains("Test description"));
        assert!(manifest.contains("[user]"));
    }

    #[tokio::test]
    async fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = scan_memory_files(temp_dir.path().to_str().unwrap()).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_scan_nonexistent_directory() {
        let result = scan_memory_files("/nonexistent/path").await;
        assert!(result.is_empty());
    }
}
