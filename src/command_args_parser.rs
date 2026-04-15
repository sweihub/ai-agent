#![allow(dead_code)]

use std::collections::HashMap;

pub fn parse_command_args(args: &[String]) -> ParsedArgs {
    ParsedArgs {
        flags: vec![],
        options: HashMap::new(),
        positional: vec![],
    }
}

pub struct ParsedArgs {
    pub flags: Vec<String>,
    pub options: HashMap<String, String>,
    pub positional: Vec<String>,
}
