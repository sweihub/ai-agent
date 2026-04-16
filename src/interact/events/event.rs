// Source: /data/home/swei/claudecode/openclaudecode/src/ink/events/event.ts
#[derive(Debug, Clone, Default)]
pub struct Event {
    did_stop_immediate_proagation: bool,
}

impl Event {
    pub fn new() -> Self {
        Self {
            did_stop_immediate_proagation: false,
        }
    }

    pub fn did_stop_immediate_propagation(&self) -> bool {
        self.did_stop_immediate_proagation
    }

    pub fn stop_immediate_propagation(&mut self) {
        self.did_stop_immediate_proagation = true;
    }
}