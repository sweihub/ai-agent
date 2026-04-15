// Source: /data/home/swei/claudecode/openclaudecode/src/services/mcp/elicitationHandler.ts
//! MCP elicitation handler for user prompts

use serde::{Deserialize, Serialize};

/// Configuration for the waiting state shown after the user opens a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationWaitingState {
    /// Button label, e.g. "Retry now" or "Skip confirmation"
    pub action_label: String,
    /// Whether to show a visible Cancel button
    #[serde(default)]
    pub show_cancel: Option<bool>,
}

/// Elicitation request event
#[derive(Debug, Clone)]
pub struct ElicitationRequestEvent {
    pub server_name: String,
    /// The JSON-RPC request ID, unique per server connection
    pub request_id: String,
    pub params: ElicitRequestParams,
    pub waiting_state: Option<ElicitationWaitingState>,
    /// Set to true by the completion notification handler when the server confirms completion
    pub completed: bool,
}

/// Elicit request parameters from MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitRequestParams {
    pub message: String,
    pub requested_schema: Option<serde_json::Value>,
    pub mode: ElicitationMode,
    pub url: Option<String>,
    pub elicitation_id: Option<String>,
}

/// Elicitation mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationMode {
    Form,
    Url,
}

/// Elicitation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitResult {
    pub action: ElicitAction,
    pub content: Option<serde_json::Value>,
}

/// Elicitation action
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitAction {
    Accept,
    Decline,
    Cancel,
}

/// Get elicitation mode from params
pub fn get_elicitation_mode(params: &ElicitRequestParams) -> &'static str {
    if params.mode == ElicitationMode::Url {
        "url"
    } else {
        "form"
    }
}

/// Find a queued elicitation event by server name and elicitationId
pub fn find_elicitation_in_queue(
    queue: &[ElicitationRequestEvent],
    server_name: &str,
    elicitation_id: &str,
) -> Option<usize> {
    queue.iter().position(|e| {
        e.server_name == server_name
            && e.params.mode == ElicitationMode::Url
            && e.params.elicitation_id.as_deref() == Some(elicitation_id)
    })
}

/// Elicitation queue state
#[derive(Debug, Clone, Default)]
pub struct ElicitationState {
    pub queue: Vec<ElicitationRequestEvent>,
}

impl ElicitationState {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// Add an elicitation to the queue
    pub fn add(&mut self, event: ElicitationRequestEvent) {
        self.queue.push(event);
    }

    /// Find and mark as completed
    pub fn mark_completed(&mut self, server_name: &str, elicitation_id: &str) -> bool {
        if let Some(idx) = find_elicitation_in_queue(&self.queue, server_name, elicitation_id) {
            self.queue[idx].completed = true;
            true
        } else {
            false
        }
    }

    /// Remove completed elicitations
    pub fn cleanup_completed(&mut self) {
        self.queue.retain(|e| !e.completed);
    }
}

/// Run elicitation hooks - returns response if handled by hooks
/// Note: Full implementation would integrate with hooks system
pub async fn run_elicitation_hooks(
    _server_name: &str,
    params: &ElicitRequestParams,
) -> Option<ElicitResult> {
    // TODO: Integrate with hooks system
    // In TypeScript:
    // const { elicitationResponse, blockingError } = await executeElicitationHooks({...})
    // if (blockingError) return { action: 'decline' }
    // if (elicitationResponse) return { action: elicitationResponse.action, content: elicitationResponse.content }

    // For now, return None to let the UI handle it
    let _ = params;
    None
}

/// Run elicitation result hooks after user responds
pub async fn run_elicitation_result_hooks(
    _server_name: &str,
    result: &ElicitResult,
) -> ElicitResult {
    // TODO: Integrate with hooks system
    // In TypeScript:
    // const { elicitationResultResponse, blockingError } = await executeElicitationResultHooks({...})
    // if (blockingError) return { action: 'decline' }

    result.clone()
}

/// Create a default cancel response
pub fn create_cancel_response() -> ElicitResult {
    ElicitResult {
        action: ElicitAction::Cancel,
        content: None,
    }
}

/// Create a decline response
pub fn create_decline_response() -> ElicitResult {
    ElicitResult {
        action: ElicitAction::Decline,
        content: None,
    }
}

/// Create an accept response with optional content
pub fn create_accept_response(content: Option<serde_json::Value>) -> ElicitResult {
    ElicitResult {
        action: ElicitAction::Accept,
        content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_elicitation_mode_form() {
        let params = ElicitRequestParams {
            message: "Test".to_string(),
            requested_schema: None,
            mode: ElicitationMode::Form,
            url: None,
            elicitation_id: None,
        };
        assert_eq!(get_elicitation_mode(&params), "form");
    }

    #[test]
    fn test_get_elicitation_mode_url() {
        let params = ElicitRequestParams {
            message: "Test".to_string(),
            requested_schema: None,
            mode: ElicitationMode::Url,
            url: Some("https://example.com".to_string()),
            elicitation_id: Some("abc123".to_string()),
        };
        assert_eq!(get_elicitation_mode(&params), "url");
    }

    #[test]
    fn test_find_elicitation_in_queue() {
        let queue = vec![
            ElicitationRequestEvent {
                server_name: "server1".to_string(),
                request_id: "1".to_string(),
                params: ElicitRequestParams {
                    message: "Test".to_string(),
                    requested_schema: None,
                    mode: ElicitationMode::Url,
                    url: None,
                    elicitation_id: Some("abc123".to_string()),
                },
                waiting_state: None,
                completed: false,
            },
        ];

        let idx = find_elicitation_in_queue(&queue, "server1", "abc123");
        assert_eq!(idx, Some(0));

        let idx = find_elicitation_in_queue(&queue, "server2", "abc123");
        assert_eq!(idx, None);
    }

    #[test]
    fn test_elicitation_state_mark_completed() {
        let mut state = ElicitationState::new();
        state.queue.push(ElicitationRequestEvent {
            server_name: "server1".to_string(),
            request_id: "1".to_string(),
            params: ElicitRequestParams {
                message: "Test".to_string(),
                requested_schema: None,
                mode: ElicitationMode::Url,
                url: None,
                elicitation_id: Some("abc123".to_string()),
            },
            waiting_state: None,
            completed: false,
        });

        assert!(state.mark_completed("server1", "abc123"));
        assert!(state.queue[0].completed);
    }
}