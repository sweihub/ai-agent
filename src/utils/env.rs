// Source: /data/home/swei/claudecode/openclaudecode/src/utils/env.ts
pub fn get_env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

#[allow(unused_variables)]
pub fn set_env(key: &str, value: &str) {
    #[cfg(not(target_os = "windows"))]
    {
        std::env::set_var(key, value);
    }
    #[cfg(target_os = "windows")]
    {
        std::env::set_var(key, value);
    }
}

#[allow(unused_variables)]
pub fn remove_env(key: &str) {
    #[cfg(not(target_os = "windows"))]
    {
        std::env::remove_var(key);
    }
    #[cfg(target_os = "windows")]
    {
        std::env::remove_var(key);
    }
}

pub fn get_all_env() -> std::collections::HashMap<String, String> {
    std::env::vars().collect()
}

pub fn is_env_set(key: &str) -> bool {
    std::env::var(key).is_ok()
}
