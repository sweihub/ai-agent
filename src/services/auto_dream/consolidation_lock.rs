pub async fn read_last_consolidated_at() -> u64 {
    0
}

pub async fn try_acquire_consolidation_lock() -> Option<u64> {
    None
}

pub async fn rollback_consolidation_lock(_prior_mtime: u64) -> Result<(), String> {
    Ok(())
}

pub async fn list_sessions_touched_since(_since_ms: u64) -> Vec<String> {
    vec![]
}

pub async fn record_consolidation() -> Result<(), String> {
    Ok(())
}
