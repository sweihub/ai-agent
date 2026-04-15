use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct BufferEntry {
    pub text: String,
    pub cursor_offset: usize,
    pub timestamp: u64,
}

pub struct InputBuffer {
    buffer: VecDeque<BufferEntry>,
    current_index: isize,
    max_size: usize,
    last_push_time: u64,
    debounce_ms: u64,
}

impl InputBuffer {
    pub fn new(max_size: usize, debounce_ms: u64) -> Self {
        Self {
            buffer: VecDeque::new(),
            current_index: -1,
            max_size,
            last_push_time: 0,
            debounce_ms,
        }
    }

    pub fn push(&mut self, text: String, cursor_offset: usize) -> bool {
        let now = now_timestamp();
        let needs_debounce = now.saturating_sub(self.last_push_time) < self.debounce_ms;

        if needs_debounce {
            return false;
        }

        self.last_push_time = now;

        if let Some(last) = self.buffer.back() {
            if last.text == text {
                return false;
            }
        }

        if self.current_index >= 0 {
            while self.buffer.len() > self.current_index as usize + 1 {
                self.buffer.pop_back();
            }
        }

        let entry = BufferEntry {
            text,
            cursor_offset,
            timestamp: now,
        };

        self.buffer.push_back(entry);

        while self.buffer.len() > self.max_size {
            self.buffer.pop_front();
        }

        let new_index = if self.current_index >= 0 {
            self.current_index + 1
        } else {
            self.buffer.len() as isize - 1
        };
        self.current_index = new_index.min(self.max_size as isize - 1).max(0);

        true
    }

    pub fn undo(&mut self) -> Option<BufferEntry> {
        if self.current_index <= 0 || self.buffer.is_empty() {
            return None;
        }

        self.current_index -= 1;
        self.buffer.get(self.current_index as usize).cloned()
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0 && self.buffer.len() > 1
    }

    pub fn redo(&mut self) -> Option<BufferEntry> {
        if self.current_index < 0 || self.buffer.is_empty() {
            return None;
        }

        let next_index = self.current_index + 1;
        if next_index >= self.buffer.len() as isize {
            return None;
        }

        self.current_index = next_index;
        self.buffer.get(self.current_index as usize).cloned()
    }

    pub fn can_redo(&self) -> bool {
        self.current_index >= 0 && self.current_index < self.buffer.len() as isize - 1
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.current_index = -1;
        self.last_push_time = 0;
    }

    pub fn get_current(&self) -> Option<&BufferEntry> {
        if self.current_index >= 0 {
            self.buffer.get(self.current_index as usize)
        } else {
            self.buffer.back()
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_buffer_push() {
        let mut buffer = InputBuffer::new(10, 0);
        assert!(buffer.push("hello".to_string(), 5));
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_input_buffer_undo() {
        let mut buffer = InputBuffer::new(10, 0);
        buffer.push("first".to_string(), 0);
        buffer.push("second".to_string(), 0);

        assert!(buffer.can_undo());
        let entry = buffer.undo();
        assert!(entry.is_some());
    }

    #[test]
    fn test_input_buffer_duplicate() {
        let mut buffer = InputBuffer::new(10, 0);
        buffer.push("same".to_string(), 0);
        assert!(!buffer.push("same".to_string(), 0));
    }
}
