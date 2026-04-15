// Source: /data/home/swei/claudecode/openclaudecode/src/commands/session/session.tsx
use crate::constants::env::system;
use crate::types::Message;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub cwd: String,
    pub model: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "messageCount")]
    pub message_count: u32,
    pub summary: Option<String>,
    pub tag: Option<String>,
}

/// Session data on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub metadata: SessionMetadata,
    pub messages: Vec<Message>,
}

/// Get the sessions directory path.
pub fn get_sessions_dir() -> PathBuf {
    let home = std::env::var(system::HOME)
        .or_else(|_| std::env::var(system::USERPROFILE))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".open-agent-sdk").join("sessions")
}

/// Get the path for a specific session.
pub fn get_session_path(session_id: &str) -> PathBuf {
    get_sessions_dir().join(session_id)
}

/// Save session to disk.
pub async fn save_session(
    session_id: &str,
    messages: Vec<Message>,
    metadata: Option<SessionMetadata>,
) -> Result<(), crate::error::AgentError> {
    let dir = get_session_path(session_id);
    fs::create_dir_all(&dir)
        .await
        .map_err(crate::error::AgentError::Io)?;

    let cwd = metadata
        .as_ref()
        .and_then(|m| Some(m.cwd.clone()))
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

    let model = metadata
        .as_ref()
        .and_then(|m| Some(m.model.clone()))
        .unwrap_or_else(|| "claude-sonnet-4-6".to_string());

    let created_at = metadata
        .as_ref()
        .and_then(|m| Some(m.created_at.clone()))
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let summary = metadata.as_ref().and_then(|m| m.summary.clone());
    let tag = metadata.as_ref().and_then(|m| m.tag.clone());

    let data = SessionData {
        metadata: SessionMetadata {
            id: session_id.to_string(),
            cwd,
            model,
            created_at: created_at.clone(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            message_count: messages.len() as u32,
            summary,
            tag,
        },
        messages,
    };

    let path = dir.join("transcript.json");
    let json = serde_json::to_string_pretty(&data).map_err(crate::error::AgentError::Json)?;
    fs::write(&path, json)
        .await
        .map_err(crate::error::AgentError::Io)?;

    Ok(())
}

/// Load session from disk.
pub async fn load_session(
    session_id: &str,
) -> Result<Option<SessionData>, crate::error::AgentError> {
    let path = get_session_path(session_id).join("transcript.json");

    match fs::read_to_string(&path).await {
        Ok(content) => {
            let data: SessionData =
                serde_json::from_str(&content).map_err(crate::error::AgentError::Json)?;
            Ok(Some(data))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(crate::error::AgentError::Io(e)),
    }
}

/// List all sessions.
pub async fn list_sessions() -> Result<Vec<SessionMetadata>, crate::error::AgentError> {
    let dir = get_sessions_dir();

    let mut entries = match fs::read_dir(&dir).await {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(crate::error::AgentError::Io(e)),
    };

    let mut sessions = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(crate::error::AgentError::Io)?
    {
        let entry_id = entry.file_name().to_string_lossy().to_string();
        if let Ok(Some(data)) = load_session(&entry_id).await {
            if let Some(metadata) = Some(data.metadata) {
                sessions.push(metadata);
            }
        }
    }

    // Sort by updatedAt descending
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(sessions)
}

/// Fork a session (create a copy with a new ID).
pub async fn fork_session(
    source_session_id: &str,
    new_session_id: Option<String>,
) -> Result<Option<String>, crate::error::AgentError> {
    let data = match load_session(source_session_id).await? {
        Some(d) => d,
        None => return Ok(None),
    };

    let fork_id = new_session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    save_session(
        &fork_id,
        data.messages,
        Some(SessionMetadata {
            id: fork_id.clone(),
            cwd: data.metadata.cwd,
            model: data.metadata.model,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            message_count: data.metadata.message_count,
            summary: Some(format!("Forked from session {}", source_session_id)),
            tag: None,
        }),
    )
    .await?;

    Ok(Some(fork_id))
}

/// Get session messages.
pub async fn get_session_messages(
    session_id: &str,
) -> Result<Vec<Message>, crate::error::AgentError> {
    match load_session(session_id).await? {
        Some(data) => Ok(data.messages),
        None => Ok(vec![]),
    }
}

/// Append a message to a session transcript.
pub async fn append_to_session(
    session_id: &str,
    message: Message,
) -> Result<(), crate::error::AgentError> {
    let mut data = match load_session(session_id).await? {
        Some(d) => d,
        None => return Ok(()),
    };

    data.messages.push(message);
    data.metadata.updated_at = chrono::Utc::now().to_rfc3339();
    data.metadata.message_count = data.messages.len() as u32;

    save_session(session_id, data.messages, Some(data.metadata)).await
}

/// Delete a session.
pub async fn delete_session(session_id: &str) -> Result<bool, crate::error::AgentError> {
    let path = get_session_path(session_id);

    match fs::remove_dir_all(&path).await {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(crate::error::AgentError::Io(e)),
    }
}

/// Get info about a specific session.
pub async fn get_session_info(
    session_id: &str,
) -> Result<Option<SessionMetadata>, crate::error::AgentError> {
    match load_session(session_id).await? {
        Some(data) => Ok(Some(data.metadata)),
        None => Ok(None),
    }
}

/// Rename a session.
pub async fn rename_session(session_id: &str, title: &str) -> Result<(), crate::error::AgentError> {
    let mut data = match load_session(session_id).await? {
        Some(d) => d,
        None => return Ok(()),
    };

    data.metadata.summary = Some(title.to_string());
    data.metadata.updated_at = chrono::Utc::now().to_rfc3339();

    save_session(session_id, data.messages, Some(data.metadata)).await
}

/// Tag a session.
pub async fn tag_session(
    session_id: &str,
    tag: Option<&str>,
) -> Result<(), crate::error::AgentError> {
    let mut data = match load_session(session_id).await? {
        Some(d) => d,
        None => return Ok(()),
    };

    data.metadata.tag = tag.map(|s| s.to_string());
    data.metadata.updated_at = chrono::Utc::now().to_rfc3339();

    save_session(session_id, data.messages, Some(data.metadata)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MessageRole;

    fn create_test_message(content: &str) -> Message {
        Message {
            role: MessageRole::User,
            content: content.to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_get_sessions_dir() {
        let dir = get_sessions_dir();
        assert!(dir.to_string_lossy().contains(".open-agent-sdk"));
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let session_id = "test-session-1";
        let messages = vec![create_test_message("Hello")];

        // Save
        save_session(session_id, messages.clone(), None)
            .await
            .unwrap();

        // Load
        let loaded = load_session(session_id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().messages.len(), 1);

        // Cleanup
        delete_session(session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_load_nonexistent_session() {
        let loaded = load_session("nonexistent-session").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_fork_session() {
        let source_id = "fork-source-test";
        let messages = vec![
            create_test_message("First"),
            Message {
                role: MessageRole::Assistant,
                content: "Response".to_string(),
                ..Default::default()
            },
        ];

        // Save original
        save_session(source_id, messages, None).await.unwrap();

        // Fork
        let fork_id = fork_session(source_id, None).await.unwrap();
        assert!(fork_id.is_some());

        // Verify fork has messages
        let fork_messages = get_session_messages(fork_id.as_ref().unwrap())
            .await
            .unwrap();
        assert_eq!(fork_messages.len(), 2);

        // Cleanup
        delete_session(source_id).await.unwrap();
        delete_session(fork_id.as_ref().unwrap()).await.unwrap();
    }

    #[tokio::test]
    async fn test_append_to_session() {
        let session_id = "append-test-session";

        // Create with initial message
        save_session(session_id, vec![create_test_message("Initial")], None)
            .await
            .unwrap();

        // Append
        append_to_session(
            session_id,
            Message {
                role: MessageRole::Assistant,
                content: "Response".to_string(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Verify
        let loaded = load_session(session_id).await.unwrap().unwrap();
        assert_eq!(loaded.messages.len(), 2);

        // Cleanup
        delete_session(session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_rename_session() {
        let session_id = "rename-test-session";
        save_session(session_id, vec![create_test_message("Test")], None)
            .await
            .unwrap();

        rename_session(session_id, "My Session").await.unwrap();

        let info = get_session_info(session_id).await.unwrap().unwrap();
        assert_eq!(info.summary, Some("My Session".to_string()));

        // Cleanup
        delete_session(session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_tag_session() {
        let session_id = "tag-test-session";
        save_session(session_id, vec![create_test_message("Test")], None)
            .await
            .unwrap();

        tag_session(session_id, Some("important")).await.unwrap();

        let info = get_session_info(session_id).await.unwrap().unwrap();
        assert_eq!(info.tag, Some("important".to_string()));

        // Cleanup
        delete_session(session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_session() {
        let session_id = "delete-test-session";
        save_session(session_id, vec![create_test_message("Test")], None)
            .await
            .unwrap();

        let result = delete_session(session_id).await.unwrap();
        assert!(result);

        // Should not exist now
        let loaded = load_session(session_id).await.unwrap();
        assert!(loaded.is_none());
    }
}
