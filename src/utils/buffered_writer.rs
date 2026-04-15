#![allow(dead_code)]

use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

pub struct BufferedWriter {
    tx: Sender<String>,
    flush_tx: Option<Sender<()>>,
}

impl BufferedWriter {
    pub fn new(
        write_fn: fn(String),
        flush_interval_ms: u64,
        max_buffer_size: usize,
        max_buffer_bytes: usize,
        immediate_mode: bool,
    ) -> Self {
        let (tx, rx) = channel();
        let (flush_tx, flush_rx) = channel();

        thread::spawn(move || {
            let mut buffer = String::new();
            let mut buffer_bytes = 0usize;
            let mut max_size = max_buffer_size;
            let mut max_bytes = max_buffer_bytes;

            loop {
                // Try to receive with timeout
                let timeout = Duration::from_millis(flush_interval_ms);
                let mut did_work = false;

                // Check for flush command
                if let Ok(_) = flush_rx.try_recv() {
                    if !buffer.is_empty() {
                        write_fn(buffer.clone());
                        buffer.clear();
                        buffer_bytes = 0;
                    }
                    did_work = true;
                }

                // Try to receive content
                while let Ok(content) = rx.try_recv() {
                    if immediate_mode {
                        write_fn(content);
                    } else {
                        buffer.push_str(&content);
                        buffer_bytes += content.len();
                        did_work = true;

                        if buffer.len() >= max_size || buffer_bytes >= max_bytes {
                            write_fn(buffer.clone());
                            buffer.clear();
                            buffer_bytes = 0;
                        }
                    }
                }

                if !did_work && !immediate_mode && !buffer.is_empty() {
                    // Only flush if there's content and we didn't do any work
                    // (meaning no new data came in)
                    write_fn(buffer.clone());
                    buffer.clear();
                    buffer_bytes = 0;
                }

                if did_work {
                    thread::sleep(Duration::from_millis(10));
                } else {
                    thread::sleep(timeout);
                }
            }
        });

        BufferedWriter {
            tx,
            flush_tx: Some(flush_tx),
        }
    }

    pub fn write(&self, content: String) {
        let _ = self.tx.send(content);
    }

    pub fn flush(&self) {
        if let Some(ref tx) = self.flush_tx {
            let _ = tx.send(());
        }
    }
}

impl Drop for BufferedWriter {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffered_writer() {
        let mut results: Vec<String> = vec![];
        let writer = BufferedWriter::new(|s| results.push(s), 1000, 100, 10000, false);
        writer.write("hello".to_string());
        writer.write("world".to_string());
        writer.flush();
        thread::sleep(Duration::from_millis(100));
    }
}
