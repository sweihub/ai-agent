#![allow(dead_code)]

use std::collections::HashMap;

pub fn validate_json_schema(_schema: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn validate_against_schema(
    _data: &str,
    _schema: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(true)
}
