use crate::constants::env::ai;
use std::path::PathBuf;
use tokio::fs;

const NAMESPACE_NOT_FOUND: &str = "namespace not found";
const CONTAINER_ID_NOT_FOUND: &str = "container ID not found";
const CONTAINER_ID_NOT_FOUND_IN_MOUNTINFO: &str = "container ID not found in mountinfo";

pub async fn get_kubernetes_namespace() -> Option<String> {
    if std::env::var(ai::USER_TYPE).ok() != Some("ant".to_string()) {
        return None;
    }

    let namespace_path = PathBuf::from("/var/run/secrets/kubernetes.io/serviceaccount/namespace");

    match fs::read_to_string(&namespace_path).await {
        Ok(content) => Some(content.trim().to_string()),
        Err(_) => Some(NAMESPACE_NOT_FOUND.to_string()),
    }
}

pub async fn get_container_id() -> Option<String> {
    if std::env::var(ai::USER_TYPE).ok() != Some("ant".to_string()) {
        return None;
    }

    let mountinfo_path = PathBuf::from("/proc/self/mountinfo");

    let mountinfo = match fs::read_to_string(&mountinfo_path).await {
        Ok(c) => c,
        Err(_) => return Some(CONTAINER_ID_NOT_FOUND.to_string()),
    };

    let content = mountinfo.trim();

    let docker_pattern = regex::Regex::new(r"/docker/containers/([0-9a-f]{64})").unwrap();
    let containerd_pattern = regex::Regex::new(r"/sandboxes/([0-9a-f]{64})").unwrap();

    for line in content.lines() {
        if let Some(caps) = docker_pattern.captures(line) {
            return Some(caps.get(1).unwrap().as_str().to_string());
        }
        if let Some(caps) = containerd_pattern.captures(line) {
            return Some(caps.get(1).unwrap().as_str().to_string());
        }
    }

    Some(CONTAINER_ID_NOT_FOUND_IN_MOUNTINFO.to_string())
}
