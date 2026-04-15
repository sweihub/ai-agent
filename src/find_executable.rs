use std::process::Command;

pub fn find_executable(exe: &str, args: Vec<String>) -> (String, Vec<String>) {
    let resolved = which_sync(exe);
    let cmd = resolved.unwrap_or_else(|| exe.to_string());
    (cmd, args)
}

fn which_sync(exe: &str) -> Option<String> {
    if cfg!(target_os = "windows") {
        Command::new("where.exe")
            .arg(exe)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
                } else {
                    None
                }
            })
    } else {
        Command::new("which")
            .arg(exe)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }
}
