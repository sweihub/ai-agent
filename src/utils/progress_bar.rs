//! Progress bar component for terminal display

/// Style for the progress bar
#[derive(Debug, Clone)]
pub struct ProgressBarStyle {
    pub fill_char: char,
    pub empty_char: char,
    pub bar_width: usize,
    pub show_percentage: bool,
    pub show_elapsed: bool,
    pub show_eta: bool,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self {
            fill_char: '█',
            empty_char: '░',
            bar_width: 40,
            show_percentage: true,
            show_elapsed: true,
            show_eta: true,
        }
    }
}

/// A progress bar for terminal display
#[derive(Debug)]
pub struct ProgressBar {
    current: usize,
    total: usize,
    style: ProgressBarStyle,
    start_time: std::time::Instant,
    message: String,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        Self {
            current: 0,
            total,
            style: ProgressBarStyle::default(),
            start_time: std::time::Instant::now(),
            message: String::new(),
        }
    }

    pub fn with_style(mut self, style: ProgressBarStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set the current progress
    pub fn set_progress(&mut self, current: usize) {
        self.current = current.min(self.total);
    }

    /// Increment progress by a value
    pub fn increment(&mut self, delta: usize) {
        self.current = (self.current + delta).min(self.total);
    }

    /// Get the current progress as a percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.current as f64 / self.total as f64) * 100.0
    }

    /// Check if the progress bar is complete
    pub fn is_complete(&self) -> bool {
        self.current >= self.total
    }

    /// Get elapsed time since creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Estimate time to completion
    pub fn eta(&self) -> Option<std::time::Duration> {
        if self.current == 0 {
            return None;
        }
        let elapsed = self.elapsed();
        let remaining = self.total - self.current;
        let per_item = elapsed / self.current as u32;
        Some(per_item * remaining as u32)
    }

    /// Render the progress bar as a string
    pub fn render(&self) -> String {
        let filled = if self.total > 0 {
            (self.current as f64 / self.total as f64 * self.style.bar_width as f64) as usize
        } else {
            self.style.bar_width
        };

        let bar: String = (0..self.style.bar_width)
            .map(|i| {
                if i < filled {
                    self.style.fill_char.to_string()
                } else {
                    self.style.empty_char.to_string()
                }
            })
            .collect();

        let mut parts = vec![];

        if !self.message.is_empty() {
            parts.push(self.message.clone());
        }

        parts.push(format!("[{}]", bar));

        if self.style.show_percentage {
            parts.push(format!("{:.1}%", self.percentage()));
        }

        if self.style.show_elapsed {
            parts.push(format!("{:?}", self.elapsed()));
        }

        if self.style.show_eta {
            if let Some(eta) = self.eta() {
                parts.push(format!("ETA: {:?}", eta));
            }
        }

        parts.join(" ")
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let mut pb = ProgressBar::new(100);
        assert_eq!(pb.percentage(), 0.0);

        pb.set_progress(50);
        assert_eq!(pb.percentage(), 50.0);

        pb.set_progress(100);
        assert!(pb.is_complete());
    }
}
