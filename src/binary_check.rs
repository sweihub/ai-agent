use std::collections::HashMap;
use std::process::Command;
use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref BINARY_CACHE: RwLock<HashMap<String, bool>> = RwLock::new(HashMap::new());
}

pub fn is_binary_installed(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return false;
    }

    if let Ok(cache) = BINARY_CACHE.read() {
        if let Some(&cached) = cache.get(trimmed) {
            return cached;
        }
    }

    let exists = which(trimmed);

    if let Ok(mut cache) = BINARY_CACHE.write() {
        cache.insert(trimmed.to_string(), exists);
    }

    exists
}

fn which(command: &str) -> bool {
    let output = if cfg!(target_os = "windows") {
        Command::new("where.exe").arg(command).output()
    } else {
        Command::new("which").arg(command).output()
    };

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub fn clear_binary_cache() {
    if let Ok(mut cache) = BINARY_CACHE.write() {
        cache.clear();
    }
}
