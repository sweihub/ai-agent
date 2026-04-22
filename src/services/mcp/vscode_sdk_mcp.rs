// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/vscodeSdkMcp.ts
//! VSCode SDK MCP module
//! Handles bidirectional communication with VSCode via MCP notifications

/// Auto mode enabled state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoModeEnabledState {
    Enabled,
    Disabled,
    OptIn,
}

impl AutoModeEnabledState {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "enabled" => Some(AutoModeEnabledState::Enabled),
            "disabled" => Some(AutoModeEnabledState::Disabled),
            "opt-in" => Some(AutoModeEnabledState::OptIn),
            _ => None,
        }
    }
}

/// Log event notification params
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEventParams {
    pub event_name: String,
    pub event_data: serde_json::Map<String, serde_json::Value>,
}

/// Experiment gates notification params
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentGatesParams {
    pub gates: serde_json::Map<String, serde_json::Value>,
}

/// File updated notification params
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUpdatedParams {
    pub file_path: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}

// Store the VSCode MCP client reference for sending notifications
// This will be set when setup_vscode_sdk_mcp is called
static VSCODE_MCP_CLIENT: std::sync::OnceLock<VscodeMcpClientState> = std::sync::OnceLock::new();

/// VSCode MCP client state (mirrors MCPServerConnection from types.ts)
#[derive(Debug, Clone)]
pub struct VscodeMcpClientState {
    pub name: String,
    pub client_type: String,
}

/// Get the VSCode MCP client state
pub fn get_vscode_mcp_client() -> Option<&'static VscodeMcpClientState> {
    VSCODE_MCP_CLIENT.get()
}

/// Check if running as ant user (USER_TYPE=ant env var, per TS process.env.USER_TYPE)
fn is_ant_user() -> bool {
    std::env::var("USER_TYPE")
        .map(|v| v == "ant")
        .unwrap_or(false)
}

/// Log for debugging
fn log_for_debugging(message: &str) {
    log::debug!("[VSCode] {}", message);
}

/// Sends a file_updated notification to the VSCode MCP server.
/// This is used to notify VSCode when files are edited or written by Claude.
pub fn notify_vscode_file_updated(
    file_path: &str,
    old_content: Option<&str>,
    new_content: Option<&str>,
) {
    // Only send notifications for ant users with an active VSCode MCP client
    if !is_ant_user() || get_vscode_mcp_client().is_none() {
        return;
    }

    // In a full implementation, this would call:
    // vscodeMcpClient.client.notification({
    //   method: 'file_updated',
    //   params: { filePath, oldContent, newContent },
    // })
    log_for_debugging(&format!(
        "file_updated notification: {} (old: {}, new: {})",
        file_path,
        old_content.map(|s| "Some").unwrap_or("None"),
        new_content.map(|s| "Some").unwrap_or("None")
    ));
}

/// Sets up the special internal VSCode MCP for bidirectional communication using notifications.
/// The `sdk_clients` parameter contains the connected MCP server connections.
pub fn setup_vscode_sdk_mcp(sdk_clients: &[VscodeMcpClientState]) {
    if !is_ant_user() {
        return;
    }

    // Find the 'claude-vscode' client from the connections (matching TS: client.name === 'claude-vscode')
    let client = sdk_clients.iter().find(|c| c.name == "claude-vscode");

    if let Some(client) = client {
        let _ = VSCODE_MCP_CLIENT.set(client.clone());
        log_for_debugging("VSCode SDK MCP initialized with claude-vscode client");
    } else {
        log_for_debugging("No claude-vscode client found, VSCode SDK MCP not initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_mode_enabled_state_from_str() {
        assert_eq!(
            AutoModeEnabledState::from_str("enabled"),
            Some(AutoModeEnabledState::Enabled)
        );
        assert_eq!(
            AutoModeEnabledState::from_str("disabled"),
            Some(AutoModeEnabledState::Disabled)
        );
        assert_eq!(
            AutoModeEnabledState::from_str("opt-in"),
            Some(AutoModeEnabledState::OptIn)
        );
        assert_eq!(AutoModeEnabledState::from_str("unknown"), None);
    }
}
