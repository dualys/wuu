use core::fmt::{self, Write};

use crate::style::{Color, ColorMode, write_colored};

/// Severity label used for concise status messages.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MessageLevel {
    /// General progress or informational output.
    Info,
    /// A completed operation.
    Success,
    /// A recoverable problem or condition needing attention.
    Warning,
    /// An operation failed.
    Error,
}

impl MessageLevel {
    const fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Success => "SUCCESS",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
        }
    }

    const fn color(self) -> Color {
        match self {
            Self::Info => Color::Cyan,
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Error => Color::Red,
        }
    }
}

/// Writes a labeled message and newline to any [`fmt::Write`] destination.
///
/// This is useful for testing or composing VCS output without requiring the
/// global terminal to be configured.
pub fn write_message<W: Write>(output: &mut W, level: MessageLevel, message: &str) -> fmt::Result {
    writeln!(output, "[{}] {}", level.label(), message)
}

/// Prints a labeled message and newline to the configured terminal.
///
/// If no terminal is configured, no output is produced.
pub fn print_message(level: MessageLevel, message: &str) {
    if let Some(term) = crate::TERMINAL.lock().as_mut() {
        let _ = write_message(term, level, message);
    }
}

/// Writes a labeled message using the selected color mode.
pub fn write_styled_message<W: Write>(
    output: &mut W,
    level: MessageLevel,
    message: &str,
    mode: ColorMode,
) -> fmt::Result {
    output.write_char('[')?;
    write_colored(output, level.label(), level.color(), mode)?;
    writeln!(output, "] {message}")
}

/// Prints a labeled message using the selected color mode.
pub fn print_styled_message(level: MessageLevel, message: &str, mode: ColorMode) {
    if let Some(term) = crate::TERMINAL.lock().as_mut() {
        let _ = write_styled_message(term, level, message, mode);
    }
}

#[cfg(test)]
mod tests {
    use super::{MessageLevel, write_message, write_styled_message};
    use crate::style::ColorMode;

    #[test]
    fn writes_labeled_messages() {
        let mut output = String::new();
        write_message(&mut output, MessageLevel::Success, "Commit created").unwrap();
        assert_eq!(output, "[SUCCESS] Commit created\n");
    }

    #[test]
    fn writes_styled_message_without_color_when_requested() {
        let mut output = String::new();
        write_styled_message(
            &mut output,
            MessageLevel::Error,
            "Checkout failed",
            ColorMode::Plain,
        )
        .unwrap();
        assert_eq!(output, "[ERROR] Checkout failed\n");
    }
}
