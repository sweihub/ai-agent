pub fn get_diff(path: &std::path::Path) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--no-color"])
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn get_staged_diff() -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--no-color"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
