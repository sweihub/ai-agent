use std::collections::VecDeque;

pub struct PriorityQueue<T> {
    items: VecDeque<T>,
    priority_fn: Box<dyn Fn(&T, &T) -> bool>,
}

impl<T> PriorityQueue<T> {
    pub fn new(priority_fn: impl Fn(&T, &T) -> bool + 'static) -> Self {
        Self {
            items: VecDeque::new(),
            priority_fn: Box::new(priority_fn),
        }
    }
    
    pub fn push(&mut self, item: T) {
        let mut inserted = false;
        for (i, existing) in self.items.iter().enumerate() {
            if (self.priority_fn)(&item, existing) {
                self.items.insert(i, item);
                inserted = true;
                break;
            }
        }
        if !inserted {
            self.items.push_back(item);
        }
    }
    
    pub fn pop(&mut Option<T> {
        self.items.pop_front()
    }
    
    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }
    
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}