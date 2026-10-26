use core::fmt::{self, Write};

/// Visual preset used to draw a table.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TableStyle {
    /// ASCII borders, suitable for minimal terminals.
    #[default]
    Ascii,
    /// Unicode single-line borders.
    Modern,
    /// No outer borders; a single rule separates the header from the rows.
    Transparent,
    /// Rounded Unicode outer corners.
    Rounded,
    /// Unicode double-line borders.
    Double,
}

#[derive(Copy, Clone)]
struct BorderGlyphs {
    top_left: &'static str,
    top_join: &'static str,
    top_right: &'static str,
    middle_left: &'static str,
    middle_join: &'static str,
    middle_right: &'static str,
    bottom_left: &'static str,
    bottom_join: &'static str,
    bottom_right: &'static str,
    horizontal: &'static str,
    vertical: &'static str,
}

impl TableStyle {
    const fn glyphs(self) -> Option<BorderGlyphs> {
        match self {
            Self::Ascii => Some(BorderGlyphs {
                top_left: "+",
                top_join: "+",
                top_right: "+",
                middle_left: "+",
                middle_join: "+",
                middle_right: "+",
                bottom_left: "+",
                bottom_join: "+",
                bottom_right: "+",
                horizontal: "-",
                vertical: "|",
            }),
            Self::Modern => Some(BorderGlyphs {
                top_left: "┌",
                top_join: "┬",
                top_right: "┐",
                middle_left: "├",
                middle_join: "┼",
                middle_right: "┤",
                bottom_left: "└",
                bottom_join: "┴",
                bottom_right: "┘",
                horizontal: "─",
                vertical: "│",
            }),
            Self::Transparent => None,
            Self::Rounded => Some(BorderGlyphs {
                top_left: "╭",
                top_join: "┬",
                top_right: "╮",
                middle_left: "├",
                middle_join: "┼",
                middle_right: "┤",
                bottom_left: "╰",
                bottom_join: "┴",
                bottom_right: "╯",
                horizontal: "─",
                vertical: "│",
            }),
            Self::Double => Some(BorderGlyphs {
                top_left: "╔",
                top_join: "╦",
                top_right: "╗",
                middle_left: "╠",
                middle_join: "╬",
                middle_right: "╣",
                bottom_left: "╚",
                bottom_join: "╩",
                bottom_right: "╝",
                horizontal: "═",
                vertical: "║",
            }),
        }
    }
}

/// A borrowed table suitable for allocation-free terminal output.
///
/// Rows may contain fewer cells than the header; missing cells are rendered
/// empty. Cells beyond the header width are ignored.
#[derive(Copy, Clone)]
pub struct Table<'a> {
    headers: &'a [&'a str],
    rows: &'a [&'a [&'a str]],
    style: TableStyle,
}

impl<'a> Table<'a> {
    /// Creates a table from column headers and borrowed rows.
    #[must_use]
    pub const fn new(headers: &'a [&'a str], rows: &'a [&'a [&'a str]]) -> Self {
        Self {
            headers,
            rows,
            style: TableStyle::Ascii,
        }
    }

    /// Selects the border style used when rendering this table.
    #[must_use]
    pub const fn with_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Writes an ASCII bordered table to any [`fmt::Write`] destination.
    ///
    /// Column widths are measured in Unicode scalar values. For correctly
    /// aligned output, use single-line cells and characters with a display
    /// width of one terminal column.
    pub fn write_to<W: Write>(&self, output: &mut W) -> fmt::Result {
        self.write_with_width(output, None)
    }

    /// Writes a table, shrinking and truncating cells to fit the requested width.
    ///
    /// Widths count Unicode scalar values, not terminal display columns.
    /// Every column retains at least one character of width, so tables with
    /// more columns than can fit at that minimum may exceed `max_width`.
    pub fn write_to_width<W: Write>(&self, output: &mut W, max_width: usize) -> fmt::Result {
        self.write_with_width(output, Some(max_width))
    }

    fn write_with_width<W: Write>(&self, output: &mut W, max_width: Option<usize>) -> fmt::Result {
        if self.headers.is_empty() {
            return Ok(());
        }

        let transparent = self.style == TableStyle::Transparent;
        if !transparent {
            self.write_border(output, max_width, BorderKind::Top)?;
        }
        self.write_row(output, self.headers, max_width)?;
        self.write_border(
            output,
            max_width,
            if transparent {
                BorderKind::Transparent
            } else {
                BorderKind::Middle
            },
        )?;
        for row in self.rows {
            self.write_row(output, row, max_width)?;
        }
        if transparent {
            Ok(())
        } else {
            self.write_border(output, max_width, BorderKind::Bottom)
        }
    }

    /// Prints the table to the configured terminal, if one is available.
    pub fn print(&self) {
        if let Some(term) = crate::TERMINAL.lock().as_mut() {
            let _ = self.write_to(term);
        }
    }

    /// Prints the table constrained to the supplied width.
    pub fn print_width(&self, max_width: usize) {
        if let Some(term) = crate::TERMINAL.lock().as_mut() {
            let _ = self.write_to_width(term, max_width);
        }
    }

    /// Prints the table using the configured terminal's current width.
    pub fn print_auto(&self) {
        if let Some(term) = crate::TERMINAL.lock().as_mut() {
            let width = term.columns();
            let _ = self.write_to_width(term, width);
        }
    }

    fn natural_column_width(&self, column: usize) -> usize {
        let mut width = self.headers[column].chars().count();
        for row in self.rows {
            if let Some(cell) = row.get(column) {
                width = width.max(cell.chars().count());
            }
        }
        width
    }

    fn column_width(&self, column: usize, max_width: Option<usize>) -> usize {
        let Some(max_width) = max_width else {
            return self.natural_column_width(column);
        };

        let count = self.headers.len();
        let mut natural_total = 0usize;
        let mut columns_after_capacity = 0usize;
        for index in 0..count {
            let natural = self.natural_column_width(index);
            natural_total = natural_total.saturating_add(natural);
            if index > column {
                columns_after_capacity =
                    columns_after_capacity.saturating_add(natural.saturating_sub(1));
            }
        }

        let overhead = if self.style == TableStyle::Transparent {
            count.saturating_sub(1).saturating_mul(2)
        } else {
            count.saturating_mul(3).saturating_add(1)
        };
        let excess = natural_total
            .saturating_add(overhead)
            .saturating_sub(max_width);
        let natural = self.natural_column_width(column);
        let shrink = excess
            .saturating_sub(columns_after_capacity)
            .min(natural.saturating_sub(1));
        natural - shrink
    }

    fn write_border<W: Write>(
        &self,
        output: &mut W,
        max_width: Option<usize>,
        kind: BorderKind,
    ) -> fmt::Result {
        let Some(glyphs) = self.style.glyphs() else {
            for column in 0..self.headers.len() {
                if column > 0 {
                    output.write_str("  ")?;
                }
                for _ in 0..self.column_width(column, max_width) {
                    output.write_str(glyphs_for_transparent(self.style))?;
                }
            }
            return output.write_char('\n');
        };

        let (left, join, right) = match kind {
            BorderKind::Top => (glyphs.top_left, glyphs.top_join, glyphs.top_right),
            BorderKind::Middle => (glyphs.middle_left, glyphs.middle_join, glyphs.middle_right),
            BorderKind::Bottom => (glyphs.bottom_left, glyphs.bottom_join, glyphs.bottom_right),
            BorderKind::Transparent => unreachable!(),
        };
        output.write_str(left)?;
        for column in 0..self.headers.len() {
            for _ in 0..self.column_width(column, max_width) + 2 {
                output.write_str(glyphs.horizontal)?;
            }
            output.write_str(if column + 1 == self.headers.len() {
                right
            } else {
                join
            })?;
        }
        output.write_char('\n')
    }

    fn write_row<W: Write>(
        &self,
        output: &mut W,
        row: &[&str],
        max_width: Option<usize>,
    ) -> fmt::Result {
        let transparent = self.style == TableStyle::Transparent;
        if !transparent {
            output.write_str(self.style.glyphs().unwrap().vertical)?;
        }
        for index in 0..self.headers.len() {
            if transparent && index > 0 {
                output.write_str("  ")?;
            }
            let width = self.column_width(index, max_width);
            let cell = row.get(index).copied().unwrap_or("");
            Self::write_cell(output, cell, width, !transparent)?;
            if !transparent {
                output.write_str(" ")?;
                output.write_str(self.style.glyphs().unwrap().vertical)?;
            }
        }
        output.write_char('\n')
    }

    fn write_cell<W: Write>(output: &mut W, cell: &str, width: usize, padded: bool) -> fmt::Result {
        if padded {
            output.write_char(' ')?;
        }
        let chars = cell.chars();
        let cell_width = chars.clone().count();
        let truncated = cell_width > width;

        if !truncated {
            for ch in chars {
                output.write_char(if ch == '\n' || ch == '\r' { ' ' } else { ch })?;
            }
            for _ in cell_width..width {
                output.write_char(' ')?;
            }
            return Ok(());
        }

        if cell.contains('/') {
            let marker_width = width.min(2);
            match marker_width {
                1 => output.write_char('~')?,
                2 => output.write_str("~/")?,
                _ => {}
            }
            for ch in cell.chars().skip(cell_width - (width - marker_width)) {
                output.write_char(if ch == '\n' || ch == '\r' { ' ' } else { ch })?;
            }
        } else {
            let content_width = width.saturating_sub(1);
            for ch in cell.chars().take(content_width) {
                output.write_char(if ch == '\n' || ch == '\r' { ' ' } else { ch })?;
            }
            output.write_char('~')?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
enum BorderKind {
    Top,
    Middle,
    Bottom,
    Transparent,
}

fn glyphs_for_transparent(style: TableStyle) -> &'static str {
    match style {
        TableStyle::Transparent => "─",
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Table, TableStyle};

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

    #[test]
    fn truncates_wide_table_to_requested_width() {
        let headers = ["STATUS", "PATH"];
        let row = ["modified", "src/very/long/path.rs"];
        let rows: [&[&str]; 1] = [&row];
        let table = Table::new(&headers, &rows);
        let mut output = String::new();

        table.write_to_width(&mut output, 22).unwrap();

        assert_eq!(
            output,
            "+----------+---------+\n\
             | STATUS   | PATH    |\n\
             +----------+---------+\n\
             | modified | ~/th.rs |\n\
             +----------+---------+\n"
        );
        assert!(output.lines().all(|line| line.chars().count() <= 22));
    }

    #[test]
    fn path_truncation_keeps_the_end_of_the_path() {
        let headers = ["PATH"];
        let row = ["src/very/long/important.rs"];
        let rows: [&[&str]; 1] = [&row];
        let table = Table::new(&headers, &rows);
        let mut output = String::new();

        table.write_to_width(&mut output, 12).unwrap();

        assert_eq!(
            output,
            "+----------+\n\
             | PATH     |\n\
             +----------+\n\
             | ~/ant.rs |\n\
             +----------+\n"
        );
    }

    #[test]
    fn renders_each_table_style() {
        let headers = ["N"];
        let row = ["x"];
        let rows: [&[&str]; 1] = [&row];
        let table = Table::new(&headers, &rows);

        let mut ascii = String::new();
        table
            .with_style(TableStyle::Ascii)
            .write_to(&mut ascii)
            .unwrap();
        assert_eq!(ascii, "+---+\n| N |\n+---+\n| x |\n+---+\n");

        let mut modern = String::new();
        table
            .with_style(TableStyle::Modern)
            .write_to(&mut modern)
            .unwrap();
        assert_eq!(modern, "┌───┐\n│ N │\n├───┤\n│ x │\n└───┘\n");

        let mut rounded = String::new();
        table
            .with_style(TableStyle::Rounded)
            .write_to(&mut rounded)
            .unwrap();
        assert_eq!(rounded, "╭───╮\n│ N │\n├───┤\n│ x │\n╰───╯\n");

        let mut double = String::new();
        table
            .with_style(TableStyle::Double)
            .write_to(&mut double)
            .unwrap();
        assert_eq!(double, "╔═══╗\n║ N ║\n╠═══╣\n║ x ║\n╚═══╝\n");

        let mut transparent = String::new();
        table
            .with_style(TableStyle::Transparent)
            .write_to(&mut transparent)
            .unwrap();
        assert_eq!(transparent, "N\n─\nx\n");
    }
}
