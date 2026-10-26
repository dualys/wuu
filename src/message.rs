use core::fmt::{self, Write};

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

#[cfg(test)]
mod tests {
    use super::{MessageLevel, write_message};

    #[test]
    fn writes_labeled_messages() {
        let mut output = String::new();
        write_message(&mut output, MessageLevel::Success, "Commit created").unwrap();
        assert_eq!(output, "[SUCCESS] Commit created\n");
    }
}
