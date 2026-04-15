// Source: /data/home/swei/claudecode/openclaudecode/src/utils/generators.ts
pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

pub fn generate_session_id() -> String {
    generate_id()
}

pub fn generate_task_id() -> String {
    generate_id()
}
