//! REPL bridge initialization

pub const TITLE_MAX_LEN: usize = 50;

pub mod init_bridge_options {
    use crate::types::Message;

    pub struct InitBridgeOptions {
        pub on_inbound_message: Option<Box<dyn Fn(serde_json::Value)>>,
        pub on_permission_response: Option<Box<dyn Fn(serde_json::Value)>>,
        pub on_interrupt: Option<Box<dyn Fn()>>,
        pub on_set_model: Option<Box<dyn Fn(Option<String>)>>,
        pub on_set_max_thinking_tokens: Option<Box<dyn Fn(Option<u64>)>>,
        pub on_set_permission_mode: Option<Box<dyn Fn(String) -> Result<(), String>>>,
        pub on_state_change: Option<Box<dyn Fn(String, Option<String>)>>,
        pub initial_messages: Option<Vec<Message>>,
        pub initial_name: Option<String>,
        pub get_messages: Option<Box<dyn Fn() -> Vec<Message>>>,
        pub previously_flushed_uuids: Option<std::collections::HashSet<String>>,
        pub perpetual: Option<bool>,
        pub outbound_only: Option<bool>,
        pub tags: Option<Vec<String>>,
    }
}
