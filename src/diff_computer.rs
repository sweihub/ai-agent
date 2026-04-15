#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_number: Option<usize>,
    pub content: String,
    pub line_type: LineType,
}

#[derive(Debug, Clone, Copy)]
pub enum LineType {
    Context,
    Added,
    Removed,
}

pub fn compute_diff(old: &str, new: &str) -> Vec<DiffLine> {
    vec![]
}
