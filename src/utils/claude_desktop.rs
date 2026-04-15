//! Claude Desktop configuration utilities.

use crate::constants::env::system;
use std::path::PathBuf;

use crate::services::mcp::types::McpServerConfig;
use crate::utils::errors::get_errno_code;
use crate::utils::json::safe_parse_json;
use crate::utils::log::log_error;
use crate::utils::platform::{get_platform, SUPPORTED_PLATFORMS};

/// Get the Claude Desktop configuration file path.
pub async fn get_claude_desktop_config_path() -> Result<String, String> {
    let platform = get_platform();

    if !SUPPORTED_PLATFORMS.contains(&platform) {
        return Err(
            "Unsupported platform: Claude Desktop integration only works on macOS and WSL."
                .to_string(),
        );
    }

    if platform == "macos" {
        let home = std::env::var(system::HOME).unwrap_or_default();
        return Ok(PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json")
            .to_string_lossy()
            .to_string());
    }

    // Windows/WSL path
    if let Some(userprofile) = std::env::var(system::USERPROFILE).ok() {
        let windows_home = userprofile.replace('\\', "/");
        let wsl_path = windows_home.replace(&['A'..='Z'].iter().collect::<String>(), "");
        let config_path = format!(
            "/mnt/c{}/AppData/Roaming/Claude/claude_desktop_config.json",
            wsl_path
        );

        if std::path::Path::new(&config_path).exists() {
            return Ok(config_path);
        }
    }

    // Try to find in /mnt/c/Users
    let users_dir = PathBuf::from("/mnt/c/Users");
    if let Ok(entries) = std::fs::read_dir(&users_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "Public"
                || name == "Default"
                || name == "Default User"
                || name == "All Users"
            {
                continue;
            }

            let potential_path = users_dir
                .join(&name)
                .join("AppData")
                .join("Roaming")
                .join("Claude")
                .join("claude_desktop_config.json");

            if potential_path.exists() {
                return Ok(potential_path.to_string_lossy().to_string());
            }
        }
    }

    Err("Could not find Claude Desktop config file in Windows. Make sure Claude Desktop is installed on Windows.".to_string())
}

/// Read MCP servers from Claude Desktop configuration.
pub async fn read_claude_desktop_mcp_servers(
) -> Result<std::collections::HashMap<String, McpServerConfig>, String> {
    let platform = get_platform();

    if !SUPPORTED_PLATFORMS.contains(&platform) {
        return Err(
            "Unsupported platform - Claude Desktop integration only works on macOS and WSL."
                .to_string(),
        );
    }

    let config_path = match get_claude_desktop_config_path().await {
        Ok(path) => path,
        Err(e) => {
            log_error(&e.into());
            return Ok(std::collections::HashMap::new());
        }
    };

    let config_content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            let code = get_errno_code(&e);
            if code == "ENOENT" {
                return Ok(std::collections::HashMap::new());
            }
            log_error(&e);
            return Ok(std::collections::HashMap::new());
        }
    };

    let config = match safe_parse_json(&config_content) {
        Some(v) => v,
        None => return Ok(std::collections::HashMap::new()),
    };

    let obj = match config.as_object() {
        Some(o) => o,
        None => return Ok(std::collections::HashMap::new()),
    };

    let mcp_servers = match obj.get("mcpServers").and_then(|v| v.as_object()) {
        Some(m) => m,
        None => return Ok(std::collections::HashMap::new()),
    };

    let mut servers = std::collections::HashMap::new();

    for (name, server_config) in mcp_servers {
        if let Some(config_obj) = server_config.as_object() {
            // Parse and validate the server config
            // For now, just add basic configs
            if let Some(command) = config_obj.get("command").and_then(|v| v.as_str()) {
                let args = config_obj
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    });

                let env = config_obj
                    .get("env")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                            .collect()
                    });

                servers.insert(
                    name.clone(),
                    McpServerConfig {
                        command: command.to_string(),
                        args,
                        env,
                    },
                );
            }
        }
    }

    Ok(servers)
}
