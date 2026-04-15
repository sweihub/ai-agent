//! Content array utilities
//!
//! Translated from openclaudecode/src/utils/contentArray.ts

use serde_json::Value;

/// Inserts a block into the content array after the last tool_result block.
/// Mutates the array in place.
///
/// # Arguments
/// * `content` - The content array to modify
/// * `block` - The block to insert
pub fn insert_block_after_tool_results(content: &mut Vec<Value>, block: Value) {
    // Find position after the last tool_result block
    let mut last_tool_result_index: isize = -1;
    for (i, item) in content.iter().enumerate() {
        if let Some(obj) = item.as_object() {
            if let Some(type_val) = obj.get("type") {
                if type_val == "tool_result" {
                    last_tool_result_index = i as isize;
                }
            }
        }
    }

    if last_tool_result_index >= 0 {
        let insert_pos = (last_tool_result_index + 1) as usize;
        content.insert(insert_pos, block);
        // Append a text continuation if the inserted block is now last
        if insert_pos == content.len() - 1 {
            let text_block = serde_json::json!({
                "type": "text",
                "text": "."
            });
            content.push(text_block);
        }
    } else {
        // No tool_result blocks — insert before the last block
        let insert_index = content.len().saturating_sub(1);
        content.insert(insert_index, block);
    }
}
