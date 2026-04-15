pub fn register_deep_link_handler(_handler: impl Fn(&str)) {}

pub fn open_deep_link(url: &str) -> Result<(), String> {
    Err("Deep link not supported".to_string())
}

pub fn is_deep_link_supported() -> bool {
    false
}
