// Source: /data/home/swei/claudecode/openclaudecode/src/utils/editor.ts
#![allow(dead_code)]

use std::collections::HashMap;

pub fn create_editor_instance() -> Result<EditorInstance, EditorError> {
    Err(EditorError::NotFound)
}

#[derive(Debug)]
pub enum EditorError {
    NotFound,
    LaunchFailed(String),
}

pub struct EditorInstance {
    pub pid: u32,
}

impl EditorInstance {
    pub fn wait(self) -> Result<(), EditorError> {
        Ok(())
    }
}
