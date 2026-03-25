use super::primitives::{error, info, success, warning};

/// Summary builder for final status output
#[derive(Debug)]
pub struct Summary {
    items: Vec<(SummaryStatus, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryStatus {
    Success,
    Warning,
    Error,
    Info,
}

impl Summary {
    /// Create a new summary
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a success item
    pub fn success(mut self, text: String) -> Self {
        self.items.push((SummaryStatus::Success, text));
        self
    }

    /// Add a warning item
    pub fn warning(mut self, text: String) -> Self {
        self.items.push((SummaryStatus::Warning, text));
        self
    }

    /// Add an error item
    pub fn error(mut self, text: String) -> Self {
        self.items.push((SummaryStatus::Error, text));
        self
    }

    /// Add an info item
    pub fn info(mut self, text: String) -> Self {
        self.items.push((SummaryStatus::Info, text));
        self
    }

    /// Render the summary
    pub fn render(&self) {
        if self.items.is_empty() {
            return;
        }

        println!();
        for (status, text) in &self.items {
            match status {
                SummaryStatus::Success => success(text),
                SummaryStatus::Warning => warning(text),
                SummaryStatus::Error => error(text),
                SummaryStatus::Info => info(text),
            }
        }
        println!();
    }
}

impl Default for Summary {
    fn default() -> Self {
        Self::new()
    }
}
