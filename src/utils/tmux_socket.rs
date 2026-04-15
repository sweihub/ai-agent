use std::process::Command;

const TMUX_COMMAND: &str = "tmux";
const CLAUDE_SOCKET_PREFIX: &str = "claude";

pub fn get_claude_socket_name() -> String {
    format!("{}-{}", CLAUDE_SOCKET_PREFIX, std::process::id())
}

pub fn check_tmux_available() -> bool {
    let result = Command::new(TMUX_COMMAND).arg("-V").output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub async fn exec_tmux(args: Vec<&str>) -> Result<(String, String, i32), String> {
    let output = Command::new(TMUX_COMMAND)
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(0);

    Ok((stdout, stderr, code))
}

pub fn get_tmux_install_instructions(platform: &str) -> String {
    match platform {
        "macos" => "Install tmux with: brew install tmux".to_string(),
        "linux" | "wsl" => "Install tmux with: sudo apt install tmux (Debian/Ubuntu) or sudo dnf install tmux (Fedora/RHEL)".to_string(),
        "windows" => "tmux is not natively available on Windows. Consider using WSL or Cygwin.".to_string(),
        _ => "Install tmux using your system package manager.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_name_format() {
        let name = get_claude_socket_name();
        assert!(name.starts_with("claude-"));
    }

    #[test]
    fn test_platform_instructions() {
        assert!(get_tmux_install_instructions("macos").contains("brew"));
        assert!(get_tmux_install_instructions("linux").contains("apt"));
    }
}
