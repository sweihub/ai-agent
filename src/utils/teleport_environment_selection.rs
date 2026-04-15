#![allow(dead_code)]

use std::collections::HashMap;

pub async fn get_current_environment() -> Result<Option<String>, Box<dyn std::error::Error>> {
    Ok(None)
}

pub async fn set_current_environment(_env_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn list_environment_selections() -> Vec<String> {
    vec![]
}

pub fn get_environment_display_name(_env_id: &str) -> String {
    String::new()
}
