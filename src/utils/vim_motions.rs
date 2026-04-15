#![allow(dead_code)]

use crate::utils::vim_types::{CommandState, FindType};

pub fn calculate_motion(
    _motion: char,
    _count: u32,
    _text: &str,
    _pos: usize,
) -> Option<(usize, usize)> {
    None
}

pub fn motion_to_offset(motion: char, _count: u32, _text: &str, _pos: usize) -> Option<usize> {
    match motion {
        'h' => Some(_pos.saturating_sub(1)),
        'l' => Some(_pos + 1),
        'j' => None,
        'k' => None,
        _ => None,
    }
}
