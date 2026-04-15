pub fn get_dynamic_env(key: &str) -> Option<String> {
    std::env::var(format!("AI_DYNAMIC_{}", key)).ok()
}

pub fn set_dynamic_env(key: &str, value: &str) {
    std::env::set_var(format!("AI_DYNAMIC_{}", key), value);
}

pub fn clear_dynamic_env(key: &str) {
    std::env::remove_var(format!("AI_DYNAMIC_{}", key));
}
