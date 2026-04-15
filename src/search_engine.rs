#![allow(dead_code)]

use std::collections::HashMap;

pub struct SearchResult {
    pub file: String,
    pub line: usize,
    pub content: String,
}

pub fn search_in_files(_query: &str, _files: &[String]) -> Vec<SearchResult> {
    vec![]
}
