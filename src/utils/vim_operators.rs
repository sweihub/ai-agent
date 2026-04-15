#![allow(dead_code)]

use crate::utils::vim_types::{CommandState, Operator, RecordedChange};

pub fn apply_operator(
    _op: Operator,
    _start: usize,
    _end: usize,
    _text: &str,
) -> Option<RecordedChange> {
    None
}

pub fn execute_operator(
    _op: Operator,
    _count: u32,
    _text: &str,
    _cursor_pos: usize,
) -> Result<(String, usize), &'static str> {
    Err("Not implemented")
}
