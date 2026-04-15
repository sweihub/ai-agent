#![allow(dead_code)]

use std::collections::HashMap;

pub fn extract_code_blocks(markdown: &str) -> Vec<CodeBlock> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
}
