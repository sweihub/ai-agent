//! Resolve file_uuid attachments on inbound bridge user messages.
//!
//! Translated from openclaudecode/src/bridge/inboundAttachments.ts
//!
//! Web composer uploads via cookie-authed /api/{org}/upload, sends file_uuid
//! alongside the message. Here we fetch each via GET /api/oauth/files/{uuid}/content
//! (oauth-authed, same store), write to ~/.ai/uploads/{sessionId}/, and
//! return @path refs to prepend. Claude's Read tool takes it from there.
//!
//! Best-effort: any failure (no token, network, non-2xx, disk) logs debug and
//! skips that attachment. The message still reaches Claude, just without @path.

use crate::constants::env::system;
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
// SCHEMA VALIDATION (STUB)
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

/// Get the uploads directory path.
fn uploads_dir() -> PathBuf {
    // This would need to be provided from the session state
    // For now, return a placeholder path
    let home = std::env::var(system::HOME).unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".ai")
        .join("uploads")
        // Session ID would be passed in or obtained from state
        .join("default")
}

// =============================================================================
// ATTACHMENT RESOLUTION (STUB)
// =============================================================================

const DOWNLOAD_TIMEOUT_MS: u64 = 30_000;

/// Debug logging helper.
fn debug(msg: &str) {
    eprintln!("[bridge:inbound-attach] {}", msg);
}

/// Resolve all attachments on an inbound message to a prefix string of
/// @path refs. Empty string if none resolved.
pub async fn resolve_inbound_attachments(_attachments: Vec<InboundAttachment>) -> String {
    // Stub: In production, this would:
    // 1. Get OAuth token from bridge config
    // 2. Fetch each file via GET /api/oauth/files/{uuid}/content
    // 3. Write to ~/.ai/uploads/{sessionId}/
    // 4. Return @path refs
    if _attachments.is_empty() {
        return String::new();
    }

    debug(&format!("resolving {} attachment(s)", _attachments.len()));
    // For now, return empty string (no attachments resolved)
    String::new()
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
