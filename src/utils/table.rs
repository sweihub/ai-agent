//! Table formatting for terminal output

use std::fmt::{Display, Formatter};

/// Alignment for table cells
#[derive(Debug, Clone, Copy, Default)]
pub enum Alignment {
    Left,
    #[default]
    Right,
    Center,
}

/// A table column definition
#[derive(Debug, Clone)]
pub struct Column {
    pub header: String,
    pub width: usize,
    pub alignment: Alignment,
}

impl Column {
    pub fn new(header: impl Into<String>, width: usize) -> Self {
        Self {
            header: header.into(),
            width,
            alignment: Alignment::default(),
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    fn align_cell(&self, content: &str) -> String {
        let content_len = content.chars().count();
        let padding = self.width.saturating_sub(content_len);

        match self.alignment {
            Alignment::Left => format!("{}{:width$}", content, "", width = padding),
            Alignment::Right => format!("{:width$}{}", "", content, width = padding),
            Alignment::Center => {
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                format!(
                    "{}{}{}",
                    " ".repeat(left_pad),
                    content,
                    " ".repeat(right_pad)
                )
            }
        }
    }
}

/// A table for terminal display
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    border_style: BorderStyle,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum BorderStyle {
    #[default]
    Ascii,
    Unicode,
    Markdown,
    None,
}

impl Table {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            border_style: BorderStyle::default(),
        }
    }

    pub fn with_border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = style;
        self
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn add_row_iter<I>(&mut self, iter: I)
    where
        I: IntoIterator,
        I::Item: Display,
    {
        let row: Vec<String> = iter.into_iter().map(|x| x.to_string()).collect();
        self.rows.push(row);
    }

    fn header_line(&self) -> String {
        match self.border_style {
            BorderStyle::Ascii => {
                let line: String = self
                    .columns
                    .iter()
                    .map(|c| "-".repeat(c.width))
                    .collect::<Vec<_>>()
                    .join("-+-");
                format!("+-{}-+", line)
            }
            BorderStyle::Unicode => {
                let line: String = self
                    .columns
                    .iter()
                    .map(|c| "─".repeat(c.width))
                    .collect::<Vec<_>>()
                    .join("┼");
                format!("┌─{}─┐", line)
            }
            BorderStyle::Markdown => {
                let line: String = self
                    .columns
                    .iter()
                    .map(|c| ":".repeat(c.width.max(3)))
                    .collect::<Vec<_>>()
                    .join("|");
                format!("|{}|", line)
            }
            BorderStyle::None => String::new(),
        }
    }

    fn row_line(&self) -> String {
        match self.border_style {
            BorderStyle::Ascii => "|".to_string(),
            BorderStyle::Unicode => "│".to_string(),
            BorderStyle::Markdown => "|".to_string(),
            BorderStyle::None => " ".to_string(),
        }
    }

    fn separator_line(&self) -> String {
        match self.border_style {
            BorderStyle::Ascii => "-+-".to_string(),
            BorderStyle::Unicode => "┼".to_string(),
            BorderStyle::Markdown => "|".to_string(),
            BorderStyle::None => " ".to_string(),
        }
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let sep = self.separator_line();
        let row_sep = self.row_line();

        // Header
        if !self.columns.is_empty() && self.border_style != BorderStyle::None {
            writeln!(f, "{}", self.header_line())?;
        }

        // Column headers
        let header_cells: Vec<String> = self
            .columns
            .iter()
            .map(|c| c.align_cell(&c.header))
            .collect();
        writeln!(f, "{}{}{}", row_sep, header_cells.join(&row_sep), row_sep)?;

        // Header separator
        if self.border_style != BorderStyle::None {
            writeln!(f, "{}", self.header_line())?;
        }

        // Rows
        for row in &self.rows {
            let cells: Vec<String> = self
                .columns
                .iter()
                .zip(row.iter())
                .map(|(c, cell)| c.align_cell(cell))
                .collect();
            writeln!(f, "{}{}{}", row_sep, cells.join(&row_sep), row_sep)?;
        }

        // Footer
        if self.border_style != BorderStyle::None {
            writeln!(f, "{}", self.header_line())?;
        }

        Ok(())
    }
}

/// Convenience function to create and populate a table
pub fn table<I>(columns: Vec<Column>, rows: I) -> Table
where
    I: IntoIterator,
    I::Item: Into<Vec<String>>,
{
    let mut table = Table::new(columns);
    for row in rows {
        table.add_row(row.into());
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table() {
        let columns = vec![
            Column::new("Name", 10),
            Column::new("Age", 5).with_alignment(Alignment::Right),
        ];

        let mut table = Table::new(columns);
        table.add_row(vec!["Alice".to_string(), "30".to_string()]);
        table.add_row(vec!["Bob".to_string(), "25".to_string()]);

        let output = table.to_string();
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
    }
}
