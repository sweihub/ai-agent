//! Hook types.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum HookEvent {
    BeforeToolUse,
    AfterToolUse,
    BeforeMessage,
    AfterMessage,
    OnError,
}

#[derive(Debug, Clone)]
pub struct Hook {
    pub event: HookEvent,
    pub handler: String,
    pub description: Option<String>,
}

impl Hook {
    pub fn new(event: HookEvent, handler: String) -> Self {
        Self {
            event,
            handler,
            description: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HookRegistry {
    pub hooks: Vec<Hook>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn register(&mut self, hook: Hook) {
        self.hooks.push(hook);
    }
}
