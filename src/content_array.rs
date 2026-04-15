use serde_json::Value;

pub fn insert_block_after_tool_results(content: &mut Vec<Value>, block: Value) {
    let mut last_tool_result_index: Option<usize> = None;

    for (i, item) in content.iter().enumerate() {
        if let Some(obj) = item.as_object() {
            if let Some(type_field) = obj.get("type") {
                if type_field == "tool_result" {
                    last_tool_result_index = Some(i);
                }
            }
        }
    }

    if let Some(idx) = last_tool_result_index {
        let insert_pos = idx + 1;
        content.insert(insert_pos, block);
        if insert_pos == content.len() - 1 {
            content.push(serde_json::json!({ "type": "text", "text": "." }));
        }
    } else if !content.is_empty() {
        let insert_index = content.len() - 1;
        content.insert(insert_index, block);
    } else {
        content.push(block);
    }
}
