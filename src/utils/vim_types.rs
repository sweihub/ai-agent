#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Change,
    Yank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindType {
    Forward(char),
    Backward(char),
    TillForward(char),
    TillBackward(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextObjScope {
    Inner,
    Around,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VimMode {
    Insert { inserted_text: String },
    Normal { command: CommandState },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandState {
    Idle,
    Count {
        digits: String,
    },
    Operator {
        op: Operator,
        count: u32,
    },
    OperatorCount {
        op: Operator,
        count: u32,
        digits: String,
    },
    OperatorFind {
        op: Operator,
        count: u32,
        find: FindType,
    },
    OperatorTextObj {
        op: Operator,
        count: u32,
        scope: TextObjScope,
    },
    Find {
        find: FindType,
        count: u32,
    },
    G {
        count: u32,
    },
    OperatorG {
        op: Operator,
        count: u32,
    },
    Replace {
        count: u32,
    },
    Indent {
        dir: char,
        count: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentState {
    pub last_change: Option<RecordedChange>,
    pub last_find: Option<FindChar>,
    pub register: String,
    pub register_is_linewise: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindChar {
    pub find_type: FindType,
    pub char: char,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordedChange {
    Insert {
        text: String,
    },
    Operator {
        op: Operator,
        motion: String,
        count: u32,
    },
    OperatorTextObj {
        op: Operator,
        obj_type: String,
        scope: TextObjScope,
        count: u32,
    },
    OperatorFind {
        op: Operator,
        find: FindType,
        char: char,
        count: u32,
    },
    Replace {
        char: char,
        count: u32,
    },
    X {
        count: u32,
    },
    ToggleCase {
        count: u32,
    },
    Indent {
        dir: char,
        count: u32,
    },
    OpenLine {
        direction: String,
    },
    Join {
        count: u32,
    },
}

pub const MAX_VIM_COUNT: u32 = 10000;

pub fn create_initial_vim_state() -> VimMode {
    VimMode::Insert {
        inserted_text: String::new(),
    }
}

pub fn create_initial_persistent_state() -> PersistentState {
    PersistentState {
        last_change: None,
        last_find: None,
        register: String::new(),
        register_is_linewise: false,
    }
}
