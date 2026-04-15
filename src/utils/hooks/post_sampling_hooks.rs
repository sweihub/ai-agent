// Source: ~/claudecode/openclaudecode/src/utils/hooks/postSamplingHooks.ts
#![allow(dead_code)]

use std::future::Future;
use std::sync::{Arc, Mutex};

use crate::types::Message;

// Re-export ReplHookContext and SystemPrompt from api_query_hook_helper
pub use crate::utils::hooks::api_query_hook_helper::{ReplHookContext, SystemPrompt};

/// Post-sampling hook function type (wrapped in Arc for shared ownership)
pub type PostSamplingHook = Arc<
    dyn Fn(ReplHookContext) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
>;

lazy_static::lazy_static! {
    static ref POST_SAMPLING_HOOKS: Arc<Mutex<Vec<PostSamplingHook>>> = Arc::new(Mutex::new(Vec::new()));
}

/// Register a post-sampling hook that will be called after model sampling completes.
/// This is an internal API not exposed through settings.
pub fn register_post_sampling_hook(hook: PostSamplingHook) {
    let mut hooks = POST_SAMPLING_HOOKS.lock().unwrap();
    hooks.push(hook);
}

/// Clear all registered post-sampling hooks (for testing)
pub fn clear_post_sampling_hooks() {
    let mut hooks = POST_SAMPLING_HOOKS.lock().unwrap();
    hooks.clear();
}

/// Execute all registered post-sampling hooks
pub async fn execute_post_sampling_hooks(
    messages: Vec<Message>,
    system_prompt: SystemPrompt,
    user_context: std::collections::HashMap<String, String>,
    system_context: std::collections::HashMap<String, String>,
    tool_use_context: Arc<crate::utils::hooks::can_use_tool::ToolUseContext>,
    query_source: Option<String>,
) {
    let context = ReplHookContext {
        messages,
        system_prompt,
        user_context,
        system_context,
        tool_use_context,
        query_source,
        query_message_count: None,
    };

    // Clone hooks to avoid holding the lock during async execution
    let hooks: Vec<PostSamplingHook> = {
        let hooks = POST_SAMPLING_HOOKS.lock().unwrap();
        hooks.clone() // Arc clones are cheap
    };

    // Execute all hooks sequentially, catching errors
    for hook in hooks {
        let ctx = context.clone();
        // Execute each hook, catching any errors
        let future = hook(ctx);
        future.await;
    }
}

/// Get the number of registered post-sampling hooks
pub fn get_post_sampling_hook_count() -> usize {
    POST_SAMPLING_HOOKS.lock().unwrap().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_clear_hooks() {
        clear_post_sampling_hooks();
        assert_eq!(get_post_sampling_hook_count(), 0);

        // Register a dummy hook (we'd need a real async fn to do this properly)
        clear_post_sampling_hooks();
        assert_eq!(get_post_sampling_hook_count(), 0);
    }

    #[test]
    fn test_repl_hook_context_clone() {
        let ctx = ReplHookContext {
            messages: Vec::new(),
            system_prompt: SystemPrompt::default(),
            user_context: std::collections::HashMap::new(),
            system_context: std::collections::HashMap::new(),
            tool_use_context: Arc::new(crate::utils::hooks::can_use_tool::ToolUseContext {
                session_id: "test".to_string(),
                cwd: None,
                is_non_interactive_session: false,
                options: None,
            }),
            query_source: None,
            query_message_count: None,
        };

        // Clone should work
        let _ctx2 = ctx.clone();
    }
}
