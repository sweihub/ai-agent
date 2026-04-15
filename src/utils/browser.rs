// Source: /data/home/swei/claudecode/openclaudecode/src/utils/browser.ts
pub fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn is_browser_available() -> bool {
    #[cfg(target_os = "macos")]
    return std::process::Command::new("open")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    #[cfg(target_os = "linux")]
    return std::process::Command::new("xdg-open")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    #[cfg(target_os = "windows")]
    return true;
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return false;
}
