use owo_colors::OwoColorize;

/// Table builder for aligned output
#[derive(Debug)]
pub struct Table {
    rows: Vec<Vec<String>>,
    headers: Option<Vec<String>>,
}

impl Table {
    /// Create a new table
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            headers: None,
        }
    }

    /// Set table headers
    #[cfg(test)]
    pub fn headers(mut self, headers: Vec<String>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Add a row to the table
    pub fn row(mut self, row: Vec<String>) -> Self {
        self.rows.push(row);
        self
    }

    /// Render the table
    pub fn render(&self) {
        if self.rows.is_empty() && self.headers.is_none() {
            return;
        }

        let mut column_widths = Vec::new();

        if let Some(headers) = &self.headers {
            column_widths = headers.iter().map(|header| header.len()).collect();
        }

        for row in &self.rows {
            for (index, cell) in row.iter().enumerate() {
                if index >= column_widths.len() {
                    column_widths.push(cell.len());
                } else {
                    column_widths[index] = column_widths[index].max(cell.len());
                }
            }
        }

        if let Some(headers) = &self.headers {
            print!("  ");
            for (index, header) in headers.iter().enumerate() {
                print!("{:<width$}", header.bold(), width = column_widths[index]);
                if index < headers.len() - 1 {
                    print!("  ");
                }
            }
            println!();
            println!();
        }

        for row in &self.rows {
            print!("  ");
            for (index, cell) in row.iter().enumerate() {
                let width = column_widths.get(index).copied().unwrap_or(0);
                print!("{:<width$}", cell, width = width);
                if index < row.len() - 1 {
                    print!("  ");
                }
            }
            println!();
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
