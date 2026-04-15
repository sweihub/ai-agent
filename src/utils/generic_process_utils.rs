use crate::constants::env::system;

pub fn get_process_id() -> u32 {
    std::process::id()
}

pub fn is_process_running(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn kill_process(pid: u32) -> Result<(), String> {
    std::process::Command::new("kill")
        .arg(pid.to_string())
        .output()
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_parent_process_id() -> Option<u32> {
    std::env::var(system::PPID).ok().and_then(|p| p.parse().ok())
}
