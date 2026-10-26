use core::fmt::{self, Write};

use crate::style::{Color, ColorMode, write_colored};

/// Borrowed unified-diff text with optional ANSI highlighting.
pub struct Diff<'a> {
    text: &'a str,
}

impl<'a> Diff<'a> {
    /// Creates a view over unified-diff text.
    #[must_use]
    pub const fn new(text: &'a str) -> Self {
        Self { text }
    }

    /// Writes diff lines, highlighting additions, removals, and hunk headers.
    pub fn write_to<W: Write>(&self, output: &mut W, mode: ColorMode) -> fmt::Result {
        for line in self.text.split_inclusive('\n') {
            let color = if line.starts_with('+') && !line.starts_with("+++") {
                Some(Color::Green)
            } else if line.starts_with('-') && !line.starts_with("---") {
                Some(Color::Red)
            } else if line.starts_with("@@") {
                Some(Color::Cyan)
            } else {
                None
            };

            if let Some(color) = color {
                write_colored(output, line, color, mode)?;
            } else {
                output.write_str(line)?;
            }
        }
        Ok(())
    }

    /// Prints the diff to the configured terminal, if one is available.
    pub fn print(&self, mode: ColorMode) {
        if let Some(term) = crate::TERMINAL.lock().as_mut() {
            let _ = self.write_to(term, mode);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Diff;
    use crate::style::ColorMode;

    #[test]
    fn plain_output_preserves_diff_text() {
        let input = "--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new\n";
        let mut output = String::new();
        Diff::new(input)
            .write_to(&mut output, ColorMode::Plain)
            .unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn ansi_output_highlights_changed_lines_not_headers() {
        let input = "--- a/file\n+++ b/file\n-old\n+new\n";
        let mut output = String::new();
        Diff::new(input)
            .write_to(&mut output, ColorMode::Ansi)
            .unwrap();
        assert_eq!(
            output,
            "--- a/file\n+++ b/file\n\x1b[31m-old\n\x1b[0m\x1b[32m+new\n\x1b[0m"
        );
    }
}
