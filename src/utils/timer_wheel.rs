pub struct TimerWheel {
    slots: Vec<Vec<Box<dyn FnOnce() + Send>>,
    current_slot: usize,
    slot_duration_ms: u64,
    total_slots: usize,
}

impl TimerWheel {
    pub fn new(slot_duration_ms: u64, total_slots: usize) -> Self {
        Self {
            slots: vec![Vec::new(); total_slots],
            current_slot: 0,
            slot_duration_ms,
            total_slots,
        }
    }
    
    pub fn schedule(&mut self, delay_ms: u64, callback: impl FnOnce() + Send + 'static) {
        let slots_ahead = ((delay_ms / self.slot_duration_ms) as usize) % self.total_slots;
        let target_slot = (self.current_slot + slots_ahead) % self.total_slots;
        self.slots[target_slot].push(Box::new(callback));
    }
    
    pub fn tick(&mut self) -> Vec<Box<dyn FnOnce() + Send>> {
        let mut callbacks = Vec::new();
        std::mem::swap(&mut callbacks, &mut self.slots[self.current_slot]);
        self.current_slot = (self.current_slot + 1) % self.total_slots;
        callbacks
    }
}