// Source: ~/claudecode/openclaudecode/src/utils/hooks/apiQueryHookHelper.ts
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::Message;

/// System prompt type - a vector of strings
pub type SystemPrompt = Vec<String>;

/// Context for REPL hooks (both post-sampling and stop hooks)
#[derive(Clone)]
pub struct ReplHookContext {
    /// Full message history including assistant responses
    pub messages: Vec<Message>,
    /// System prompt
    pub system_prompt: SystemPrompt,
    /// User context key-value pairs
    pub user_context: std::collections::HashMap<String, String>,
    /// System context key-value pairs
    pub system_context: std::collections::HashMap<String, String>,
    /// Tool use context
    pub tool_use_context: Arc<crate::utils::hooks::can_use_tool::ToolUseContext>,
    /// Query source identifier
    pub query_source: Option<String>,
    /// Optional: message count for API queries
    pub query_message_count: Option<usize>,
}

/// Configuration for an API query hook
pub struct ApiQueryHookConfig<TResult> {
    /// Query source name
    pub name: String,
    /// Whether this hook should run
    pub should_run: Box<dyn Fn(&ReplHookContext) -> std::pin::Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync>,
    /// Build the complete message list to send to the API
    pub build_messages: Box<dyn Fn(&ReplHookContext) -> Vec<Message> + Send + Sync>,
    /// Optional: override system prompt (defaults to context.system_prompt)
    pub system_prompt: Option<SystemPrompt>,
    /// Optional: whether to use tools from context (defaults to true)
    pub use_tools: Option<bool>,
    /// Parse the response content into a result
    pub parse_response: Box<dyn Fn(&str, &ReplHookContext) -> TResult + Send + Sync>,
    /// Log the result
    pub log_result: Box<dyn Fn(ApiQueryResult<TResult>, &ReplHookContext) + Send + Sync>,
    /// Get the model to use (lazy loaded)
    pub get_model: Box<dyn Fn(&ReplHookContext) -> String + Send + Sync>,
}

/// Result of an API query hook execution
pub enum ApiQueryResult<TResult> {
    Success {
        query_name: String,
        result: TResult,
        message_id: String,
        model: String,
        uuid: String,
    },
    Error {
        query_name: String,
        error: Box<dyn std::error::Error + Send + Sync>,
        uuid: String,
    },
}

/// Create an API query hook from the given configuration.
/// Returns an async function that executes the hook when called.
pub fn create_api_query_hook<TResult: 'static>(
    config: ApiQueryHookConfig<TResult>,
) -> Box<dyn Fn(ReplHookContext) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync> {
    let config = Arc::new(config);
    Box::new(move |context: ReplHookContext| {
        let config = config.clone();
        Box::pin(async move {
            let should_run = (config.should_run)(&context).await;
            if !should_run {
                return;
            }

            let uuid = Uuid::new_v4().to_string();

            // Build messages using the config's build_messages function
            let messages = (config.build_messages)(&context);
            // Note: we can't mutate context directly in Rust; the caller
            // would need to handle query_message_count tracking externally

            // Use config's system prompt if provided, otherwise use context's
            let system_prompt = config
                .system_prompt
                .clone()
                .unwrap_or_else(|| context.system_prompt.clone());

            // Use config's tools preference (defaults to true = use context tools)
            // In Rust, tool access would be through the tool_use_context

            // Get model (lazy loaded)
            let model = (config.get_model)(&context);

            // Make API call - this would use the actual query function
            // The TS version calls queryModelWithoutStreaming
            let response_result = query_model_without_streaming_impl(
                &messages,
                &system_prompt,
                &model,
                &context,
            )
            .await;

            match response_result {
                Ok(response) => {
                    // Extract text content from response
                    let content = extract_text_content(&response.content).trim().to_string();

                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (config.parse_response)(&content, &context)
                    }));

                    match result {
                        Ok(parsed_result) => {
                            (config.log_result)(
                                ApiQueryResult::Success {
                                    query_name: config.name.clone(),
                                    result: parsed_result,
                                    message_id: response.message_id,
                                    model,
                                    uuid,
                                },
                                &context,
                            );
                        }
                        Err(err) => {
                            let error = if let Some(s) = err.downcast_ref::<String>() {
                                Box::new(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    s.clone(),
                                ))
                            } else if let Some(s) = err.downcast_ref::<&str>() {
                                Box::new(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    s.to_string(),
                                ))
                            } else {
                                Box::new(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Unknown panic in parse_response",
                                ))
                            };
                            (config.log_result)(
                                ApiQueryResult::Error {
                                    query_name: config.name.clone(),
                                    error,
                                    uuid,
                                },
                                &context,
                            );
                        }
                    }
                }
                Err(error) => {
                    log_error(&format!("API query hook error: {}", error));
                }
            }
        })
    })
}

/// Internal struct for API response
struct ApiResponse {
    message_id: String,
    content: String,
}

/// Simulated queryModelWithoutStreaming implementation.
/// In practice, this would call the actual API query function.
async fn query_model_without_streaming_impl(
    _messages: &[Message],
    _system_prompt: &SystemPrompt,
    _model: &str,
    _context: &ReplHookContext,
) -> Result<ApiResponse, Box<dyn std::error::Error + Send + Sync>> {
    // This is a placeholder - the actual implementation would call
    // the API query function from the crate's services module
    Err("query_model_without_streaming not implemented in port".into())
}

/// Extract text content from a message content string or structured content
fn extract_text_content(content: &str) -> &str {
    content
}

/// Log an error (simplified version of logError)
fn log_error(msg: &str) {
    log::error!("{}", msg);
}

/// Create a system prompt from a list of strings
pub fn as_system_prompt(parts: Vec<&str>) -> SystemPrompt {
    parts.iter().map(|s| s.to_string()).collect()
}
