//! Screenshot clipboard utilities.

use std::process::Command;

/// Copy screenshot to clipboard
pub fn copy_screenshot_to_clipboard(image_path: &str) -> Result<(), String> {
    // Try using xclip on Linux
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("xclip")
            .args([
                "-selection",
                "clipboard",
                "-t",
                "image/png",
                "-i",
                image_path,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        return Ok(());
    }

    // Try using pbcopy on macOS
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .args([
                "-e",
                &format!("set the clipboard to (read file \"{}\" as PNG)", image_path),
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        return Ok(());
    }

    Err("Unsupported platform".to_string())
}

/// Take a screenshot
pub fn take_screenshot() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("gnome-screenshot")
            .args(["-f", "/tmp/screenshot.png"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            return Ok("/tmp/screenshot.png".to_string());
        }

        // Try import
        let output = Command::new("import")
            .args(["-window", "root", "/tmp/screenshot.png"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            return Ok("/tmp/screenshot.png".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("screencapture")
            .args(["-i", "/tmp/screenshot.png"])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            return Ok("/tmp/screenshot.png".to_string());
        }
    }

    Err("Failed to take screenshot".to_string())
}
