#![allow(dead_code)]

use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

lazy_static! {
    static ref EXCLUDED_FILENAMES: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("package-lock.json");
        s.insert("yarn.lock");
        s.insert("pnpm-lock.yaml");
        s.insert("bun.lockb");
        s.insert("bun.lock");
        s.insert("composer.lock");
        s.insert("gemfile.lock");
        s.insert("cargo.lock");
        s.insert("poetry.lock");
        s.insert("pipfile.lock");
        s.insert("shrinkwrap.json");
        s.insert("npm-shrinkwrap.json");
        s
    };
    static ref EXCLUDED_EXTENSIONS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert(".lock");
        s.insert(".min.js");
        s.insert(".min.css");
        s.insert(".min.html");
        s.insert(".bundle.js");
        s.insert(".bundle.css");
        s.insert(".generated.ts");
        s.insert(".generated.js");
        s.insert(".d.ts");
        s
    };
    static ref EXCLUDED_DIRECTORIES: Vec<&'static str> = vec![
        "/dist/",
        "/build/",
        "/out/",
        "/output/",
        "/node_modules/",
        "/vendor/",
        "/vendored/",
        "/third_party/",
        "/third-party/",
        "/external/",
        "/.next/",
        "/.nuxt/",
        "/.svelte-kit/",
        "/coverage/",
        "/__pycache__/",
        "/.tox/",
        "/venv/",
        "/.venv/",
        "/target/release/",
        "/target/debug/",
    ];
    static ref EXCLUDED_FILENAME_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)^.*\.min\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*-min\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*\.bundle\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*\.generated\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*\.gen\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*\.auto\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*_generated\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*_gen\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*\.pb\.(go|js|ts|py|rb)$").unwrap(),
        Regex::new(r"(?i)^.*_pb2?\.py$").unwrap(),
        Regex::new(r"(?i)^.*\.pb\.h$").unwrap(),
        Regex::new(r"(?i)^.*\.grpc\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*\.swagger\.[a-z]+$").unwrap(),
        Regex::new(r"(?i)^.*\.openapi\.[a-z]+$").unwrap(),
    ];
}

pub fn is_generated_file(file_path: &str) -> bool {
    let file_name = file_path.split('/').last().unwrap_or("").to_lowercase();
    let ext = file_path
        .rsplit('.')
        .next()
        .map(|e| format!(".{}", e))
        .unwrap_or_default()
        .to_lowercase();

    if EXCLUDED_FILENAMES.contains(&file_name.as_str()) {
        return true;
    }

    if EXCLUDED_EXTENSIONS.contains(&ext.as_str())
        || EXCLUDED_EXTENSIONS.contains(&format!(".{}", file_name).to_lowercase().as_str())
    {
        return true;
    }

    for dir in EXCLUDED_DIRECTORIES.iter() {
        if file_path.contains(*dir) {
            return true;
        }
    }

    for pattern in EXCLUDED_FILENAME_PATTERNS.iter() {
        if pattern.is_match(&file_name) {
            return true;
        }
    }

    false
}

pub fn filter_generated_files(files: &[String]) -> Vec<String> {
    files
        .iter()
        .filter(|f| !is_generated_file(f))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_generated_file() {
        assert!(is_generated_file("/dist/bundle.js"));
        assert!(is_generated_file("package-lock.json"));
        assert!(!is_generated_file("/src/main.rs"));
    }

    #[test]
    fn test_filter_generated_files() {
        let files = vec![
            "src/main.ts".to_string(),
            "package-lock.json".to_string(),
            "dist/app.js".to_string(),
        ];
        let filtered = filter_generated_files(&files);
        assert_eq!(filtered.len(), 1);
    }
}
