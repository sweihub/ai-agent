#![allow(dead_code)]

use crate::utils::vim_types::CommandState;

pub fn transition_from_idle(key: char) -> Option<CommandState> {
    match key {
        'd' | 'c' | 'y' => Some(CommandState::Operator {
            op: match key {
                'd' => crate::utils::vim_types::Operator::Delete,
                'c' => crate::utils::vim_types::Operator::Change,
                'y' => crate::utils::vim_types::Operator::Yank,
                _ => crate::utils::vim_types::Operator::Delete,
            },
            count: 1,
        }),
        '0'..='9' => Some(CommandState::Count {
            digits: key.to_string(),
        }),
        'f' | 'F' | 't' | 'T' => Some(CommandState::Find {
            find: crate::utils::vim_types::FindType::Forward(key),
            count: 1,
        }),
        'g' => Some(CommandState::G { count: 1 }),
        'r' => Some(CommandState::Replace { count: 1 }),
        '>' | '<' => Some(CommandState::Indent { dir: key, count: 1 }),
        _ => None,
    }
}

pub fn transition_from_count(state: &CommandState, key: char) -> Option<CommandState> {
    match state {
        CommandState::Count { digits } if key.is_ascii_digit() => {
            let mut new_digits = digits.clone();
            new_digits.push(key);
            Some(CommandState::Count { digits: new_digits })
        }
        _ => None,
    }
}
