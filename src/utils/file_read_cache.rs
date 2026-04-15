use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static FILE_READ_CACHE: std::sync::LazyLock<Mutex<HashMap<String, (String, Instant)>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

const CACHE_TTL_SECS: u64 = 5;

pub fn get_cached_file_content(path: &str) -> Option<String> {
    let cache = FILE_READ_CACHE.lock().ok()?;
    let (content, instant) = cache.get(path)?;
    if instant.elapsed() > Duration::from_secs(CACHE_TTL_SECS) {
        return None;
    }
    Some(content.clone())
}

pub fn set_cached_file_content(path: &str, content: String) {
    if let Ok(mut cache) = FILE_READ_CACHE.lock() {
        cache.insert(path.to_string(), (content, Instant::now()));
    }
}

pub fn clear_file_read_cache() {
    if let Ok(mut cache) = FILE_READ_CACHE.lock() {
        cache.clear();
    }
}
