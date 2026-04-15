// Source: ~/claudecode/openclaudecode/src/utils/messagePredicates.ts

/// Tool result messages share type:'user' with human turns; the discriminant
/// is the optional tool_use_result field. Four PRs (#23977, #24016, #24022,
/// #24025) independently fixed miscounts from checking type==='user' alone.
pub fn is_human_turn(message: &crate::types::message::Message) -> bool {
    message.msg_type == "user"
        && !message.is_meta
        && message.tool_use_result.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::Message;

    #[test]
    fn test_is_human_turn() {
        let msg = Message {
            msg_type: "user".to_string(),
            is_meta: false,
            tool_use_result: None,
            content: "hello".to_string(),
        };
        assert!(is_human_turn(&msg));
    }

    #[test]
    fn test_not_human_turn_when_meta() {
        let msg = Message {
            msg_type: "user".to_string(),
            is_meta: true,
            tool_use_result: None,
            content: "hello".to_string(),
        };
        assert!(!is_human_turn(&msg));
    }

    #[test]
    fn test_not_human_turn_when_tool_result() {
        let msg = Message {
            msg_type: "user".to_string(),
            is_meta: false,
            tool_use_result: Some("result".to_string()),
            content: "hello".to_string(),
        };
        assert!(!is_human_turn(&msg));
    }
}
