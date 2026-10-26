use core::fmt::{self, Write};

/// Allocation-free terminal progress bar.
///
/// Progress is clamped to the configured total. A zero total is treated as
/// complete and displays 100%.
pub struct ProgressBar<'a> {
    total: usize,
    position: usize,
    width: usize,
    message: &'a str,
}

impl<'a> ProgressBar<'a> {
    /// Creates a progress bar with a 20-character interior.
    #[must_use]
    pub const fn new(total: usize) -> Self {
        Self {
            total,
            position: 0,
            width: 20,
            message: "",
        }
    }

    /// Creates a progress bar with a label and a 20-character interior.
    #[must_use]
    pub const fn with_message(total: usize, message: &'a str) -> Self {
        Self {
            total,
            position: 0,
            width: 20,
            message,
        }
    }

    /// Sets the number of completed units.
    pub fn set_position(&mut self, position: usize) {
        self.position = position.min(self.total);
    }

    /// Advances the progress by one unit, without exceeding the total.
    pub fn inc(&mut self) {
        self.set_position(self.position.saturating_add(1));
    }

    /// Sets the number of characters in the bar interior.
    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    /// Returns the current position and total.
    #[must_use]
    pub const fn position(&self) -> (usize, usize) {
        (self.position, self.total)
    }

    /// Writes the current progress on the active terminal line.
    ///
    /// Use this when refreshing the progress bar; it begins with a carriage
    /// return so each update replaces the previous display.
    pub fn write_to<W: Write>(&self, output: &mut W) -> fmt::Result {
        output.write_char('\r')?;
        self.write_contents(output)
    }

    /// Writes the current progress followed by a newline.
    pub fn write_finish<W: Write>(&self, output: &mut W) -> fmt::Result {
        self.write_to(output)?;
        output.write_char('\n')
    }

    /// Prints the current progress to the configured terminal, if available.
    pub fn print(&self) {
        if let Some(term) = crate::TERMINAL.lock().as_mut() {
            let _ = self.write_to(term);
        }
    }

    /// Prints the current progress and ends the terminal line.
    pub fn finish(&self) {
        if let Some(term) = crate::TERMINAL.lock().as_mut() {
            let _ = self.write_finish(term);
        }
    }

    fn write_contents<W: Write>(&self, output: &mut W) -> fmt::Result {
        if !self.message.is_empty() {
            write!(output, "{} ", self.message)?;
        }

        let percent = if self.total == 0 {
            100
        } else {
            ((self.position as u128 * 100) / self.total as u128) as usize
        };
        let filled = if self.total == 0 {
            self.width
        } else {
            (self.position as u128 * self.width as u128 / self.total as u128) as usize
        };

        output.write_char('[')?;
        for _ in 0..filled {
            output.write_char('=')?;
        }
        for _ in filled..self.width {
            output.write_char(' ')?;
        }
        write!(output, "] {percent:3}%")
    }
}

#[cfg(test)]
mod tests {
    use super::ProgressBar;

    #[test]
    fn renders_progress_and_clamps_to_total() {
        let mut bar = ProgressBar::with_message(4, "Fetching");
        bar.set_width(8);
        bar.set_position(3);
        let mut output = String::new();

        bar.write_to(&mut output).unwrap();

        assert_eq!(output, "\rFetching [======  ]  75%");
    }

    #[test]
    fn increments_and_finishes_with_newline() {
        let mut bar = ProgressBar::new(2);
        bar.set_width(4);
        bar.inc();
        bar.inc();
        bar.inc();
        let mut output = String::new();

        bar.write_finish(&mut output).unwrap();

        assert_eq!(bar.position(), (2, 2));
        assert_eq!(output, "\r[====] 100%\n");
    }

    #[test]
    fn zero_total_is_complete() {
        let mut bar = ProgressBar::new(0);
        bar.set_width(3);
        let mut output = String::new();

        bar.write_to(&mut output).unwrap();

        assert_eq!(output, "\r[===] 100%");
    }
}
