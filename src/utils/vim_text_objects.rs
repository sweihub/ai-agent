#![allow(dead_code)]

use std::collections::HashMap;

pub fn get_text_object_range(_text: &str, _obj_type: char, _scope: &str) -> Option<(usize, usize)> {
    None
}

pub fn is_text_object_key(key: char) -> bool {
    matches!(
        key,
        'w' | 'W' | '"' | '\'' | '`' | '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>'
    )
}
