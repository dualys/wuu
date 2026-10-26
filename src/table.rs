use core::fmt::{self, Write};

/// A borrowed table suitable for allocation-free terminal output.
///
/// Rows may contain fewer cells than the header; missing cells are rendered
/// empty. Cells beyond the header width are ignored.
pub struct Table<'a> {
    headers: &'a [&'a str],
    rows: &'a [&'a [&'a str]],
}

impl<'a> Table<'a> {
    /// Creates a table from column headers and borrowed rows.
    #[must_use]
    pub const fn new(headers: &'a [&'a str], rows: &'a [&'a [&'a str]]) -> Self {
        Self { headers, rows }
    }

    /// Writes an ASCII bordered table to any [`fmt::Write`] destination.
    ///
    /// Column widths are measured in Unicode scalar values. For correctly
    /// aligned output, use single-line cells and characters with a display
    /// width of one terminal column.
    pub fn write_to<W: Write>(&self, output: &mut W) -> fmt::Result {
        if self.headers.is_empty() {
            return Ok(());
        }

        self.write_border(output)?;
        self.write_row(output, self.headers)?;
        self.write_border(output)?;
        for row in self.rows {
            self.write_row(output, row)?;
        }
        self.write_border(output)
    }

    /// Prints the table to the configured terminal, if one is available.
    pub fn print(&self) {
        if let Some(term) = crate::TERMINAL.lock().as_mut() {
            let _ = self.write_to(term);
        }
    }

    fn column_width(&self, column: usize) -> usize {
        let mut width = self.headers[column].chars().count();
        for row in self.rows {
            if let Some(cell) = row.get(column) {
                width = width.max(cell.chars().count());
            }
        }
        width
    }

    fn write_border<W: Write>(&self, output: &mut W) -> fmt::Result {
        output.write_char('+')?;
        for column in 0..self.headers.len() {
            for _ in 0..self.column_width(column) + 2 {
                output.write_char('-')?;
            }
            output.write_char('+')?;
        }
        output.write_char('\n')
    }

    fn write_row<W: Write>(&self, output: &mut W, row: &[&str]) -> fmt::Result {
        output.write_char('|')?;
        for index in 0..self.headers.len() {
            let width = self.column_width(index);
            let cell = row.get(index).copied().unwrap_or("");
            let cell_width = cell.chars().count();
            write!(output, " {cell}")?;
            for _ in cell_width..width {
                output.write_char(' ')?;
            }
            output.write_str(" |")?;
        }
        output.write_char('\n')
    }
}

#[cfg(test)]
mod tests {
    use super::Table;

    #[test]
    fn writes_aligned_table_and_empty_missing_cells() {
        let headers = ["CHANGE", "STATUS", "FILES"];
        let first = ["M", "modified", "src/lib.rs"];
        let second = ["?", "untracked"];
        let rows: [&[&str]; 2] = [&first, &second];
        let table = Table::new(&headers, &rows);
        let mut output = String::new();

        table.write_to(&mut output).unwrap();

        assert_eq!(
            output,
            "+--------+-----------+------------+\n\
             | CHANGE | STATUS    | FILES      |\n\
             +--------+-----------+------------+\n\
             | M      | modified  | src/lib.rs |\n\
             | ?      | untracked |            |\n\
             +--------+-----------+------------+\n"
        );
    }

    #[test]
    fn writes_nothing_without_columns() {
        let table = Table::new(&[], &[]);
        let mut output = String::new();
        table.write_to(&mut output).unwrap();
        assert!(output.is_empty());
    }
}
