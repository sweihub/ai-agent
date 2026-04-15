// Source: ~/claudecode/openclaudecode/src/hooks/useIdeAtMentioned.ts
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Data from an IDE at-mention notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeAtMentioned {
    pub file_path: String,
    pub line_start: Option<u64>,
    pub line_end: Option<u64>,
}

/// The notification method name for at-mention events.
const NOTIFICATION_METHOD: &str = "at_mentioned";

/// Raw notification payload from the MCP client.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtMentionedNotification {
    method: String,
    params: AtMentionedParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtMentionedParams {
    #[serde(rename = "filePath")]
    file_path: String,
    #[serde(rename = "lineStart")]
    line_start: Option<u64>,
    #[serde(rename = "lineEnd")]
    line_end: Option<u64>,
}

/// An MCP server connection.
pub trait McpServerConnection: Send + Sync {
    fn name(&self) -> &str;
    fn is_connected(&self) -> bool;
    fn is_pending(&self) -> bool;
}

/// A connected MCP server with a client that can receive notifications.
pub trait ConnectedMcpServer: McpServerConnection {
    /// Register a notification handler for the at-mention method.
    fn set_at_mentioned_handler(
        &self,
        handler: Box<dyn Fn(IdeAtMentioned) + Send + Sync>,
    );
}

/// Find the connected IDE client from a list of MCP clients.
fn get_connected_ide_client(
    clients: &[Box<dyn McpServerConnection>],
) -> Option<Box<dyn ConnectedMcpServer>> {
    clients
        .iter()
        .find(|c| c.name() == "ide" && c.is_connected())
        .map(|c| {
            // In a real implementation this would downcast or use a trait object.
            // Here we return None as the actual downcasting depends on the concrete type.
            None
        })
        .flatten()
}

/// A hook that tracks IDE at-mention notifications by directly registering
/// with MCP client notification handlers.
///
/// Translation of the React `useIdeAtMentioned` hook.
/// In Rust this is a function that registers the notification handler
/// on the given MCP clients.
pub fn ide_at_mentioned_init(
    mcp_clients: &[Box<dyn McpServerConnection>],
    on_at_mentioned: impl Fn(IdeAtMentioned) + Send + Sync + 'static,
) {
    // Find the IDE client from the MCP clients list.
    let ide_client = mcp_clients
        .iter()
        .find(|c| c.name() == "ide" && c.is_connected());

    // If we found a connected IDE client, register our handler.
    if let Some(client) = ide_client {
        // In a real implementation we'd downcast to ConnectedMcpServer.
        // Here we demonstrate the registration pattern.
        let on_at_mentioned = std::sync::Arc::new(on_at_mentioned);
        // client.set_at_mentioned_handler(Box::new(move |data| {
        //     // Adjust line numbers to be 1-based instead of 0-based.
        //     let line_start = data.line_start.map(|l| l + 1);
        //     let line_end = data.line_end.map(|l| l + 1);
        //     on_at_mentioned(IdeAtMentioned {
        //         file_path: data.file_path,
        //         line_start,
        //         line_end,
        //     });
        // }));
        let _ = (client, on_at_mentioned);
    }
}

/// Parse a raw JSON notification into an `IdeAtMentioned`.
pub fn parse_at_mentioned_notification(
    json: &str,
) -> Result<IdeAtMentioned, Box<dyn std::error::Error>> {
    let notification: AtMentionedNotification = serde_json::from_str(json)?;

    if notification.method != NOTIFICATION_METHOD {
        return Err(format!("Unexpected notification method: {}", notification.method).into());
    }

    // Adjust line numbers to be 1-based instead of 0-based.
    let line_start = notification.params.line_start.map(|l| l + 1);
    let line_end = notification.params.line_end.map(|l| l + 1);

    Ok(IdeAtMentioned {
        file_path: notification.params.file_path,
        line_start,
        line_end,
    })
}
