#![allow(dead_code)]

use std::process::Command;

pub fn find_executable(exe: &str, args: Vec<String>) -> (String, Vec<String>) {
    // Try to find executable using 'which' on Unix, 'where' on Windows
    let output = if cfg!(target_os = "windows") {
        Command::new("where").arg(exe).output()
    } else {
        Command::new("which").arg(exe).output()
    };

    let cmd = match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| exe.to_string()),
        _ => exe.to_string(),
    };

    (cmd, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_executable() {
        let (cmd, args) = find_executable("ls", vec!["-la".to_string()]);
        assert!(!cmd.is_empty());
        assert_eq!(args, vec!["-la".to_string()]);
    }
}
