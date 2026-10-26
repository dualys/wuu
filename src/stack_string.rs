use core::fmt::{self, Write};

/// Fixed-capacity text buffer stored inline without heap allocation.
///
/// The capacity `N` is measured in bytes. Text written beyond the remaining
/// capacity is truncated.
pub struct StackString<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StackString<N> {
    /// Creates an empty buffer with a capacity of `N` bytes.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    /// Returns the buffered bytes as a string.
    ///
    /// Returns an empty string if truncation leaves the buffer with invalid UTF-8.
    #[must_use]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Default for StackString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Write for StackString<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remaining = N.saturating_sub(self.len);
        let to_copy = bytes.len().min(remaining);
        self.buf[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;
        Ok(())
    }
}

/// Formats text into a [`StackString`] with a capacity of 256 bytes.
///
/// The returned buffer may contain truncated text if the formatted output
/// exceeds its capacity.
#[macro_export]
macro_rules! format_stack {
    ($($arg:tt)*) => {{
        let mut s = $crate::StackString::<256>::new();
        let _ = core::fmt::write(&mut s, format_args!($($arg)*));
        s
    }};
}
