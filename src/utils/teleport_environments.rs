#![allow(dead_code)]

use std::collections::HashMap;

pub struct TeleportEnvironment {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

pub async fn fetch_environments() -> Result<Vec<TeleportEnvironment>, Box<dyn std::error::Error>> {
    Ok(vec![])
}

pub async fn create_environment(
    _name: &str,
    _description: Option<&str>,
) -> Result<TeleportEnvironment, Box<dyn std::error::Error>> {
    Err("Not implemented".into())
}

pub async fn delete_environment(_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
