// Source: /data/home/swei/claudecode/openclaudecode/src/query/transitions.ts
use std::collections::HashMap;

pub struct QueryTransitions {
    pub from_state: String,
    pub to_state: String,
    pub event: String,
}

impl QueryTransitions {
    pub fn new(from: &str, to: &str, event: &str) -> Self {
        Self {
            from_state: from.to_string(),
            to_state: to.to_string(),
            event: event.to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.from_state.is_empty() && !self.to_state.is_empty()
    }
}

pub struct StateMachine {
    pub current_state: String,
    pub transitions: HashMap<String, Vec<QueryTransitions>>,
}

impl StateMachine {
    pub fn new(initial_state: &str) -> Self {
        Self {
            current_state: initial_state.to_string(),
            transitions: HashMap::new(),
        }
    }

    pub fn add_transition(&mut self, transition: QueryTransitions) {
        self.transitions
            .entry(transition.from_state.clone())
            .or_insert_with(Vec::new)
            .push(transition);
    }

    pub fn transition(&mut self, event: &str) -> bool {
        if let Some(transitions) = self.transitions.get(&self.current_state) {
            for t in transitions {
                if t.event == event {
                    self.current_state = t.to_state.clone();
                    return true;
                }
            }
        }
        false
    }
}
