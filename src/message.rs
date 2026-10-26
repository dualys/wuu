use core::fmt::{self, Arguments, Write};

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

/// Writes an OpenRC-style successful status line.
///
/// `width` is the desired total line width in characters. Use single-column
/// text for descriptions and statuses to align reliably in terminals.
pub fn write_ok<W: Write>(
    output: &mut W,
    description: &str,
    width: usize,
    mode: ColorMode,
) -> fmt::Result {
    write_status(output, '*', description, "ok", width, Color::Green, mode)
}

/// Writes an OpenRC-style failed status line.
///
/// `width` is the desired total line width in characters. Use single-column
/// text for descriptions and statuses to align reliably in terminals.
pub fn write_ko<W: Write>(
    output: &mut W,
    description: &str,
    width: usize,
    mode: ColorMode,
) -> fmt::Result {
    write_status(output, '!', description, "ko", width, Color::Red, mode)
}

/// Writes an OpenRC-style successful status line with custom status text.
///
/// `width` is the desired total line width in characters. Use single-column
/// text for descriptions and statuses to align reliably in terminals.
pub fn write_ok_custom<W: Write>(
    output: &mut W,
    description: &str,
    status: &str,
    width: usize,
    mode: ColorMode,
) -> fmt::Result {
    write_status(output, '*', description, status, width, Color::Green, mode)
}

/// Writes an OpenRC-style failed status line with custom status text.
///
/// `width` is the desired total line width in characters. Use single-column
/// text for descriptions and statuses to align reliably in terminals.
pub fn write_ko_custom<W: Write>(
    output: &mut W,
    description: &str,
    status: &str,
    width: usize,
    mode: ColorMode,
) -> fmt::Result {
    write_status(output, '!', description, status, width, Color::Red, mode)
}

/// Prints an OpenRC-style successful status line to the configured terminal.
pub fn ok(description: &str) {
    if let Some(term) = crate::TERMINAL.lock().as_mut() {
        let width = term.columns();
        let _ = write_ok(term, description, width, ColorMode::Ansi);
    }
}

/// Prints an OpenRC-style failed status line to the configured terminal.
pub fn ko(description: &str) {
    if let Some(term) = crate::TERMINAL.lock().as_mut() {
        let width = term.columns();
        let _ = write_ko(term, description, width, ColorMode::Ansi);
    }
}

/// Prints an OpenRC-style successful status line with custom status text.
pub fn ok_custom(description: &str, status: &str) {
    if let Some(term) = crate::TERMINAL.lock().as_mut() {
        let width = term.columns();
        let _ = write_ok_custom(term, description, status, width, ColorMode::Ansi);
    }
}

/// Prints an OpenRC-style failed status line with custom status text.
pub fn ko_custom(description: &str, status: &str) {
    if let Some(term) = crate::TERMINAL.lock().as_mut() {
        let width = term.columns();
        let _ = write_ko_custom(term, description, status, width, ColorMode::Ansi);
    }
}

fn write_status<W: Write>(
    output: &mut W,
    marker: char,
    description: &str,
    status: &str,
    width: usize,
    color: Color,
    mode: ColorMode,
) -> fmt::Result {
    let marker_text = if marker == '*' { "*" } else { "!" };
    write_colored(output, marker_text, color, mode)?;
    write!(output, " {description} ")?;

    let fixed_width = 8 + description.chars().count() + status.chars().count();
    for _ in 0..width.saturating_sub(fixed_width) {
        output.write_char('.')?;
    }

    output.write_str(" [ ")?;
    write_colored(output, status, color, mode)?;
    output.write_str(" ]\n")
}

/// Prints a formatted, ANSI-colored message without allocating.
///
/// This function is intended for the exported level macros.
#[doc(hidden)]
pub fn print_formatted_message(level: MessageLevel, args: Arguments<'_>) {
    if let Some(term) = crate::TERMINAL.lock().as_mut() {
        let _ = write_formatted_message(term, level, args);
    }
}

fn write_formatted_message<W: Write>(
    output: &mut W,
    level: MessageLevel,
    args: Arguments<'_>,
) -> fmt::Result {
    output.write_char('[')?;
    write_colored(output, level.label(), level.color(), ColorMode::Ansi)?;
    output.write_str("] ")?;
    output.write_fmt(args)?;
    output.write_char('\n')
}

/// Prints an informational message to the configured terminal.
///
/// Formatting follows [`core::format_args!`]; no heap allocation is needed.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::message::print_formatted_message(
            $crate::MessageLevel::Info,
            core::format_args!($($arg)*),
        )
    };
}

/// Prints a success message to the configured terminal.
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        $crate::message::print_formatted_message(
            $crate::MessageLevel::Success,
            core::format_args!($($arg)*),
        )
    };
}

/// Prints a warning message to the configured terminal.
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {
        $crate::message::print_formatted_message(
            $crate::MessageLevel::Warning,
            core::format_args!($($arg)*),
        )
    };
}

/// Prints an error message to the configured terminal.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::message::print_formatted_message(
            $crate::MessageLevel::Error,
            core::format_args!($($arg)*),
        )
    };
}

#[cfg(test)]
mod tests {
    use super::{
        MessageLevel, write_formatted_message, write_ko, write_ko_custom, write_message, write_ok,
        write_ok_custom, write_styled_message,
    };
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

    #[test]
    fn formats_level_message_without_allocation() {
        let mut output = String::new();
        write_formatted_message(
            &mut output,
            MessageLevel::Success,
            format_args!("Created commit {}", "abc123"),
        )
        .unwrap();
        assert_eq!(output, "[\x1b[32mSUCCESS\x1b[0m] Created commit abc123\n");
    }

    #[test]
    fn writes_openrc_style_status_lines() {
        let mut output = String::new();
        write_ok(&mut output, "Build", 25, ColorMode::Plain).unwrap();
        write_ko(&mut output, "Network", 25, ColorMode::Plain).unwrap();
        write_ok_custom(&mut output, "Service", "started", 25, ColorMode::Plain).unwrap();
        write_ko_custom(&mut output, "Config", "invalid", 25, ColorMode::Plain).unwrap();

        assert_eq!(
            output,
            "* Build .......... [ ok ]\n\
! Network ........ [ ko ]\n\
* Service ... [ started ]\n\
! Config .... [ invalid ]\n"
        );
    }

    #[test]
    fn status_line_does_not_underflow_for_long_descriptions() {
        let mut output = String::new();
        write_ok(&mut output, "A very long description", 8, ColorMode::Plain).unwrap();
        assert_eq!(output, "* A very long description  [ ok ]\n");
    }

    #[test]
    fn custom_status_supports_empty_text_and_minimum_width() {
        let mut output = String::new();
        write_ok_custom(&mut output, "Ready", "", 0, ColorMode::Plain).unwrap();
        assert_eq!(output, "* Ready  [  ]\n");
    }

    #[test]
    fn ansi_status_colors_match_the_result_and_reset() {
        let mut output = String::new();
        write_ok_custom(&mut output, "Service", "ready", 27, ColorMode::Ansi).unwrap();
        write_ko_custom(&mut output, "Network", "offline", 27, ColorMode::Ansi).unwrap();

        assert_eq!(
            output,
            "\x1b[32m*\x1b[0m Service ....... [ \x1b[32mready\x1b[0m ]\n\
\x1b[31m!\x1b[0m Network ..... [ \x1b[31moffline\x1b[0m ]\n"
        );
    }

    #[test]
    fn status_line_matches_requested_width_for_single_column_text() {
        let mut output = String::new();
        write_ko_custom(&mut output, "Network", "offline", 30, ColorMode::Plain).unwrap();

        assert_eq!(output.lines().next().unwrap().chars().count(), 30);
    }
}
