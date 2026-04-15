//! Image store utilities
//!
//! Store and retrieve images from disk for clipboard integration.

use crate::constants::env::{ai, system};
use std::collections::HashMap;
use std::path::PathBuf;

const IMAGE_STORE_DIR: &str = "image-cache";
const MAX_STORED_IMAGE_PATHS: usize = 200;

/// In-memory cache of stored image paths
static STORED_IMAGE_PATHS: once_cell::sync::Lazy<std::sync::Mutex<HashMap<u64, String>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(HashMap::new()));

/// Get the image store directory for the current session
fn get_image_store_dir() -> PathBuf {
    // TODO: Get session ID properly
    let session_id =
        std::env::var(ai::CODE_SESSION_ID).unwrap_or_else(|_| "default".to_string());

    // Get config home dir
    let config_home = std::env::var(ai::CONFIG_HOME)
        .or_else(|_| std::env::var(ai::CLAUDE_CONFIG_HOME))
        .or_else(|_| std::env::var(system::HOME).map(|h| format!("{}/.ai", h)))
        .unwrap_or_else(|_| "~/.ai".to_string());

    PathBuf::from(config_home)
        .join(IMAGE_STORE_DIR)
        .join(session_id)
}

/// Ensure the image store directory exists
async fn ensure_image_store_dir() -> std::io::Result<()> {
    let dir = get_image_store_dir();
    tokio::fs::create_dir_all(dir).await
}

/// Get the file path for an image by ID
fn get_image_path(image_id: u64, media_type: &str) -> PathBuf {
    let extension = media_type.split('/').nth(1).unwrap_or("png");
    get_image_store_dir().join(format!("{}.{}", image_id, extension))
}

/// Evict oldest entries if at capacity
fn evict_oldest_if_at_cap() {
    let mut map = STORED_IMAGE_PATHS.lock().unwrap();
    while map.len() >= MAX_STORED_IMAGE_PATHS {
        if let Some(oldest) = map.keys().next().copied() {
            map.remove(&oldest);
        } else {
            break;
        }
    }
}

/// Get the stored image path by ID
pub fn get_stored_image_path(image_id: u64) -> Option<String> {
    STORED_IMAGE_PATHS.lock().unwrap().get(&image_id).cloned()
}

/// Clear the in-memory cache of stored image paths
pub fn clear_stored_image_paths() {
    STORED_IMAGE_PATHS.lock().unwrap().clear();
}

/// Store an image from pasted content to disk.
pub async fn store_image(image_id: u64, media_type: &str, content: &str) -> Option<String> {
    ensure_image_store_dir().await.ok()?;

    let image_path = get_image_path(image_id, media_type);

    // Decode base64 to bytes
    let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, content).ok()?;

    // Write to file
    tokio::fs::write(&image_path, data).await.ok()?;

    evict_oldest_if_at_cap();

    let path_str = image_path.to_string_lossy().to_string();
    STORED_IMAGE_PATHS
        .lock()
        .unwrap()
        .insert(image_id, path_str.clone());

    Some(path_str)
}

/// Store all images from a map of ID to content
pub async fn store_images(images: &HashMap<u64, (&str, &str)>) -> HashMap<u64, String> {
    let mut path_map = HashMap::new();

    for (&id, (media_type, content)) in images {
        if let Some(path) = store_image(id, media_type, content).await {
            path_map.insert(id, path);
        }
    }

    path_map
}

/// Clean up old image cache directories from previous sessions
pub async fn cleanup_old_image_caches() {
    let base_dir = {
        let config_home = std::env::var(ai::CONFIG_HOME)
            .or_else(|_| std::env::var(ai::CLAUDE_CONFIG_HOME))
            .or_else(|_| std::env::var(system::HOME).map(|h| format!("{}/.ai", h)))
            .unwrap_or_else(|_| "~/.ai".to_string());

        PathBuf::from(config_home).join(IMAGE_STORE_DIR)
    };

    let current_session_id =
        std::env::var(ai::CODE_SESSION_ID).unwrap_or_else(|_| "default".to_string());

    // Read directory entries
    let mut entries = match tokio::fs::read_dir(&base_dir).await {
        Ok(e) => e,
        Err(_) => return,
    };

    // Use a simple for loop to collect entries
    let mut entries_vec: Vec<tokio::fs::DirEntry> = Vec::new();

    // Collect entries
    loop {
        match entries.next_entry().await {
            Ok(Some(entry)) => entries_vec.push(entry),
            Ok(None) => break,
            Err(_) => break,
        }
    }

    // Process each session directory
    for entry in entries_vec {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == current_session_id {
            continue;
        }

        let path = base_dir.join(&name);
        let _ = tokio::fs::remove_dir_all(path).await;
    }
}
