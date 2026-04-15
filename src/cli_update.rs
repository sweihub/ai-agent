#![allow(dead_code)]

pub async fn run_update_check() -> Result<bool, Box<dyn std::error::Error>> {
    Ok(false)
}

pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
