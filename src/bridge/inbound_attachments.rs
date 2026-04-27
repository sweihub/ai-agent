// Source: ~/claudecode/openclaudecode/src/bridge/inboundAttachments.ts
//! Resolve file_uuid attachments on inbound bridge user messages.
//!
//! Web composer uploads via cookie-authed /api/{org}/upload, sends file_uuid
//! alongside the message. Here we fetch each via GET /api/oauth/files/{uuid}/content
//! (oauth-authed, same store), write to ~/.ai/uploads/{sessionId}/, and
//! return @path refs to prepend. Claude's Read tool takes it from there.
//!
//! Best-effort: any failure (no token, network, non-2xx, disk) logs debug and
//! skips that attachment. The message still reaches Claude, just without @path.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// =============================================================================
// TYPES
// =============================================================================

/// An inbound attachment from a bridge message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundAttachment {
    pub file_uuid: String,
    pub file_name: String,
}

// =============================================================================
// SCHEMA VALIDATION
// =============================================================================

/// Pull file_attachments off a loosely-typed inbound message.
pub fn extract_inbound_attachments(msg: &serde_json::Value) -> Vec<InboundAttachment> {
    let file_attachments = match msg.get("file_attachments") {
        Some(v) => v,
        None => return Vec::new(),
    };

    if let Some(arr) = file_attachments.as_array() {
        arr.iter()
            .filter_map(|item| {
                let file_uuid = item.get("file_uuid")?.as_str()?.to_string();
                let file_name = item.get("file_name")?.as_str()?.to_string();
                Some(InboundAttachment {
                    file_uuid,
                    file_name,
                })
            })
            .collect()
    } else {
        Vec::new()
    }
}

// =============================================================================
// FILE SANITIZATION
// =============================================================================

/// Strip path components and keep only filename-safe chars.
/// file_name comes from the network (web composer), so treat it as untrusted.
fn sanitize_file_name(name: &str) -> String {
    let base = std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);

    let sanitized: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        "attachment".to_string()
    } else {
        sanitized
    }
}

// =============================================================================
// DIRECTORY HELPERS
// =============================================================================

/// Get the uploads directory path for the current session.
fn uploads_dir() -> PathBuf {
    let config_home = crate::utils::env_utils::get_claude_config_home_dir();
    let session_id = crate::bootstrap::state::get_session_id();
    PathBuf::from(config_home)
        .join("uploads")
        .join(session_id)
}

// =============================================================================
// ATTACHMENT RESOLUTION
// =============================================================================

const DOWNLOAD_TIMEOUT_MS: u64 = 30_000;

/// Debug logging helper.
fn debug(msg: &str) {
    log::debug!("[bridge:inbound-attach] {}", msg);
}

/// Fetch + write one attachment. Returns the absolute path on success,
/// undefined on any failure.
async fn resolve_one(att: &InboundAttachment) -> Option<String> {
    let token = match crate::bridge::bridge_config::get_bridge_access_token() {
        Some(t) => t,
        None => {
            debug("skip: no oauth token");
            return None;
        }
    };

    let base_url = crate::bridge::bridge_config::get_bridge_base_url();
    let encoded_uuid = urlencoding::encode(&att.file_uuid).into_owned();
    let url = format!("{}/api/oauth/files/{}/content", base_url, encoded_uuid);

    let client = reqwest::Client::new();
    let mut req_builder = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_millis(DOWNLOAD_TIMEOUT_MS));

    let response = match req_builder.send().await {
        Ok(r) => r,
        Err(e) => {
            debug(&format!("fetch {} threw: {}", att.file_uuid, e));
            return None;
        }
    };

    if response.status() != 200 {
        debug(&format!(
            "fetch {} failed: status={}",
            att.file_uuid,
            response.status()
        ));
        return None;
    }

    let data = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            debug(&format!("fetch {} read failed: {}", att.file_uuid, e));
            return None;
        }
    };

    // uuid-prefix makes collisions impossible across messages and within one
    // (same filename, different files). 8 chars is enough — this isn't security.
    let safe_name = sanitize_file_name(&att.file_name);
    let prefix = att
        .file_uuid
        .chars()
        .take(8)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let prefix = if prefix.is_empty() {
        uuid::Uuid::new_v4().to_string().chars().take(8).collect()
    } else {
        prefix
    };

    let dir = uploads_dir();
    let out_path = dir.join(format!("{}-{}", prefix, safe_name));

    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        debug(&format!("create dir {} failed: {}", dir.display(), e));
        return None;
    }

    if let Err(e) = tokio::fs::write(&out_path, &data).await {
        debug(&format!("write {} failed: {}", out_path.display(), e));
        return None;
    }

    debug(&format!(
        "resolved {} → {} ({} bytes)",
        att.file_uuid,
        out_path.display(),
        data.len()
    ));

    Some(out_path.to_string_lossy().to_string())
}

/// Resolve all attachments on an inbound message to a prefix string of
/// @path refs. Empty string if none resolved.
pub async fn resolve_inbound_attachments(attachments: Vec<InboundAttachment>) -> String {
    if attachments.is_empty() {
        return String::new();
    }

    debug(&format!("resolving {} attachment(s)", attachments.len()));

    // Resolve all attachments concurrently
    let futures: Vec<_> = attachments.iter().map(resolve_one).collect();
    let paths: Vec<Option<String>> = futures_util::future::join_all(futures).await;

    let ok: Vec<String> = paths.into_iter().filter_map(|p| p).collect();
    if ok.is_empty() {
        return String::new();
    }

    // Quoted form — extractAtMentionedFiles truncates unquoted @refs at the
    // first space, which breaks any home dir with spaces.
    ok.iter()
        .map(|p| format!("\"@{}\"", p))
        .collect::<Vec<_>>()
        .join(" ")
        + " "
}

/// Prepend @path refs to content, whichever form it's in.
/// Targets the LAST text block.
pub fn prepend_path_refs(content: &str, prefix: &str) -> String {
    if prefix.is_empty() {
        return content.to_string();
    }
    format!("{}{}", prefix, content)
}

/// Convenience: extract + resolve + prepend.
/// No-op when the message has no file_attachments field.
pub async fn resolve_and_prepend(msg: &serde_json::Value, content: &str) -> String {
    let attachments = extract_inbound_attachments(msg);
    if attachments.is_empty() {
        return content.to_string();
    }
    let prefix = resolve_inbound_attachments(attachments).await;
    prepend_path_refs(content, &prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_inbound_attachments_empty() {
        let msg = serde_json::json!({});
        assert!(extract_inbound_attachments(&msg).is_empty());
    }

    #[test]
    fn test_extract_inbound_attachments_valid() {
        let msg = serde_json::json!({
            "file_attachments": [
                {"file_uuid": "abc-123", "file_name": "test.png"}
            ]
        });
        let atts = extract_inbound_attachments(&msg);
        assert_eq!(atts.len(), 1);
        assert_eq!(atts[0].file_uuid, "abc-123");
        assert_eq!(atts[0].file_name, "test.png");
    }

    #[test]
    fn test_sanitize_file_name_basic() {
        assert_eq!(sanitize_file_name("hello.txt"), "hello.txt");
        assert_eq!(sanitize_file_name("path/to/file.txt"), "file.txt");
    }

    #[test]
    fn test_sanitize_file_name_special_chars() {
        let result = sanitize_file_name("hello world.txt");
        assert_eq!(result, "hello_world.txt");
    }

    #[test]
    fn test_sanitize_file_name_empty() {
        let result = sanitize_file_name("///");
        // Path::new("///").file_name() is None, falls back to original "///"
        // which gets sanitized to "___"
        assert_eq!(result, "___");
    }

    #[test]
    fn test_prepend_path_refs_empty_prefix() {
        assert_eq!(prepend_path_refs("hello", ""), "hello");
    }

    #[test]
    fn test_prepend_path_refs_with_prefix() {
        assert_eq!(prepend_path_refs("hello", "@file.txt "), "@file.txt hello");
    }
}
