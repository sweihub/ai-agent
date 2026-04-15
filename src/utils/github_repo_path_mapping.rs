use std::collections::HashMap;
use std::sync::Mutex;

static REPO_MAPPING: std::sync::LazyLock<Mutex<HashMap<String, String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn map_github_repo_to_local_path(repo: &str) -> Option<String> {
    REPO_MAPPING.lock().unwrap().get(repo).cloned()
}

pub fn set_github_repo_mapping(repo: &str, local_path: &str) {
    REPO_MAPPING
        .lock()
        .unwrap()
        .insert(repo.to_string(), local_path.to_string());
}

pub fn clear_mappings() {
    REPO_MAPPING.lock().unwrap().clear();
}
