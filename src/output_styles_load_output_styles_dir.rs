#![allow(dead_code)]

use std::collections::HashMap;

pub struct OutputStyle {
    pub name: String,
    pub foreground: Option<String>,
    pub background: Option<String>,
}

pub fn load_output_styles_dir(_path: &str) -> Vec<OutputStyle> {
    vec![]
}
