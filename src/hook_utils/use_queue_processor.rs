use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct QueueItem<T: Clone> {
    pub data: T,
    pub timestamp: u64,
    pub priority: i32,
}

impl<T: Clone> Clone for QueueItem<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            timestamp: self.timestamp,
            priority: self.priority,
        }
    }
}

pub struct QueueProcessor<T: Clone> {
    queue: Arc<Mutex<VecDeque<QueueItem<T>>>>,
    max_size: usize,
    processing: Arc<Mutex<bool>>,
}

impl<T: Clone> QueueProcessor<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
            processing: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn enqueue(&self, data: T, priority: i32) -> bool {
        let mut queue = self.queue.lock().await;

        if queue.len() >= self.max_size {
            return false;
        }

        let item = QueueItem {
            data,
            timestamp: now_timestamp(),
            priority,
        };

        let mut insert_index = None;
        for i in 0..queue.len() {
            if queue[i].priority < priority {
                insert_index = Some(i);
                break;
            }
        }

        if let Some(idx) = insert_index {
            queue.insert(idx, item);
        } else {
            queue.push_back(item);
        }

        true
    }

    pub async fn dequeue(&self) -> Option<T> {
        let mut queue = self.queue.lock().await;
        queue.pop_front().map(|item| item.data)
    }

    pub async fn peek(&self) -> Option<T> {
        let queue = self.queue.lock().await;
        queue.front().cloned().map(|item| item.data)
    }

    pub async fn len(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.lock().await;
        queue.is_empty()
    }

    pub async fn clear(&self) {
        let mut queue = self.queue.lock().await;
        queue.clear();
    }

    pub async fn is_processing(&self) -> bool {
        let processing = self.processing.lock().await;
        *processing
    }

    pub async fn set_processing(&self, value: bool) {
        let mut processing = self.processing.lock().await;
        *processing = value;
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

    #[tokio::test]
    async fn test_queue_processor_enqueue() {
        let processor = QueueProcessor::new(10);
        assert!(processor.enqueue("item1", 1).await);
        assert_eq!(processor.len().await, 1);
    }

    #[tokio::test]
    async fn test_queue_processor_priority() {
        let processor = QueueProcessor::new(10);
        processor.enqueue("low", 1).await;
        processor.enqueue("high", 10).await;

        let first = processor.dequeue().await;
        assert_eq!(first, Some("high".to_string()));
    }

    #[tokio::test]
    async fn test_queue_processor_full() {
        let processor = QueueProcessor::new(2);
        processor.enqueue("1", 1).await;
        processor.enqueue("2", 1).await;
        assert!(!processor.enqueue("3", 1).await);
    }
}
