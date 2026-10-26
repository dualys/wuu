use core::fmt::{self, Write};

/// Whether terminal output should include ANSI color sequences.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ColorMode {
    /// Plain text without escape sequences.
    Plain,
    /// ANSI color escape sequences.
    Ansi,
}

/// Basic ANSI foreground colors.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Color {
    /// Green, commonly used for additions and success.
    Green,
    /// Red, commonly used for removals and errors.
    Red,
    /// Yellow, commonly used for warnings and changes.
    Yellow,
    /// Cyan, commonly used for headings and metadata.
    Cyan,
}

impl Color {
    const fn code(self) -> &'static str {
        match self {
            Self::Green => "\x1b[32m",
            Self::Red => "\x1b[31m",
            Self::Yellow => "\x1b[33m",
            Self::Cyan => "\x1b[36m",
        }
    }
}

/// Writes text with optional ANSI color, resetting the style afterward.
pub fn write_colored<W: Write>(
    output: &mut W,
    text: &str,
    color: Color,
    mode: ColorMode,
) -> fmt::Result {
    if mode == ColorMode::Ansi {
        output.write_str(color.code())?;
        output.write_str(text)?;
        output.write_str("\x1b[0m")
    } else {
        output.write_str(text)
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, ColorMode, write_colored};

    #[test]
    fn color_can_be_disabled() {
        let mut output = String::new();
        write_colored(&mut output, "+file", Color::Green, ColorMode::Plain).unwrap();
        assert_eq!(output, "+file");
    }

    #[test]
    fn ansi_style_is_reset_after_text() {
        let mut output = String::new();
        write_colored(&mut output, "+file", Color::Green, ColorMode::Ansi).unwrap();
        assert_eq!(output, "\x1b[32m+file\x1b[0m");
    }
}
