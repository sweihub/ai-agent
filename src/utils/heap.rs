pub struct Heap<T: Ord> {
    data: Vec<T>,
}

impl<T: Ord> Heap<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    pub fn push(&mut self, item: T) {
        self.data.push(item);
        self.bubble_up(self.data.len() - 1);
    }
    
    pub fn pop(&mut Option<T> {
        if self.data.is_empty() {
            return None;
        }
        let min = self.data.first().cloned();
        let last = self.data.pop().unwrap();
        if !self.data.is_empty() {
            self.data[0] = last;
            self.bubble_down(0);
        }
        min
    }
    
    pub fn peek(&self) -> Option<&T> {
        self.data.first()
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    fn bubble_up(&mut self, mut idx: usize) {
        while idx > 0 {
            let parent = (idx - 1) / 2;
            if self.data[idx] < self.data[parent] {
                self.data.swap(idx, parent);
                idx = parent;
            } else {
                break;
            }
        }
    }
    
    fn bubble_down(&mut self, mut idx: usize) {
        loop {
            let left = 2 * idx + 1;
            let right = 2 * idx + 2;
            let mut smallest = idx;
            
            if left < self.data.len() && self.data[left] < self.data[smallest] {
                smallest = left;
            }
            if right < self.data.len() && self.data[right] < self.data[smallest] {
                smallest = right;
            }
            
            if smallest != idx {
                self.data.swap(idx, smallest);
                idx = smallest;
            } else {
                break;
            }
        }
    }
}

impl<T: Ord> Default for Heap<T> {
    fn default() -> Self {
        Self::new()
    }
}