use crate::PromptKey;

const fn ascii_key_table() -> [Option<PromptKey>; 128] {
    let mut table = [None; 128];
    let mut byte = b' ';
    while byte <= b'~' {
        table[byte as usize] = Some(PromptKey::Character(byte as char));
        byte += 1;
    }
    table[8] = Some(PromptKey::Backspace);
    table[10] = Some(PromptKey::Enter);
    table[13] = Some(PromptKey::Enter);
    table[27] = Some(PromptKey::Escape);
    table[127] = Some(PromptKey::Backspace);
    table
}

/// ASCII byte-to-key mapping for prompt input.
///
/// Printable ASCII bytes map to `Character`; backspace, line feed, carriage
/// return, escape, and delete map to their corresponding prompt keys. Other
/// control bytes are unmapped.
pub const ASCII_KEY_TABLE: [Option<PromptKey>; 128] = ascii_key_table();

/// Incremental decoder for ASCII terminal input and common ANSI key sequences.
///
/// Feed bytes from a serial console, UART, or OS keyboard-to-ASCII keymap using
/// [`push`](Self::push). An isolated Escape byte is reported by
/// [`flush_escape`](Self::flush_escape) after the host's escape timeout.
#[derive(Default)]
pub struct AsciiKeyDecoder {
    escape_state: u8,
}

impl AsciiKeyDecoder {
    /// Creates a decoder with no pending input.
    #[must_use]
    pub const fn new() -> Self {
        Self { escape_state: 0 }
    }

    /// Feeds one ASCII/ANSI byte and returns a decoded prompt key, if complete.
    pub fn push(&mut self, byte: u8) -> Option<PromptKey> {
        match self.escape_state {
            0 if byte == 0x1b => {
                self.escape_state = 1;
                None
            }
            0 => PromptKey::from_ascii(byte),
            1 if byte == b'[' || byte == b'O' => {
                self.escape_state = 2;
                None
            }
            1 => {
                self.escape_state = 0;
                Some(PromptKey::Escape)
            }
            2 => {
                self.escape_state = 0;
                match byte {
                    b'A' => Some(PromptKey::Up),
                    b'B' => Some(PromptKey::Down),
                    _ => None,
                }
            }
            _ => {
                self.escape_state = 0;
                None
            }
        }
    }

    /// Resolves a pending Escape byte when no further sequence byte arrives.
    pub fn flush_escape(&mut self) -> Option<PromptKey> {
        if self.escape_state == 1 {
            self.escape_state = 0;
            Some(PromptKey::Escape)
        } else {
            None
        }
    }
}

#[cfg(all(feature = "linux-keyboard", target_os = "linux"))]
pub mod linux {
    use std::io;
    use std::time::Duration;

    use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

    use crate::PromptKey;

    /// Linux terminal keyboard reader that enables raw mode for interactive prompts.
    ///
    /// Dropping the reader attempts to restore the terminal's previous mode.
    pub struct LinuxKeyboard {
        active: bool,
    }

    impl LinuxKeyboard {
        /// Enables raw mode so keys are returned immediately without Enter.
        pub fn new() -> io::Result<Self> {
            enable_raw_mode()?;
            Ok(Self { active: true })
        }

        /// Waits up to `timeout` for a key press.
        ///
        /// Returns `Ok(None)` on timeout or for unsupported key events.
        pub fn read_key(&mut self, timeout: Duration) -> io::Result<Option<PromptKey>> {
            if !event::poll(timeout)? {
                return Ok(None);
            }
            let Event::Key(event) = event::read()? else {
                return Ok(None);
            };
            if !matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                return Ok(None);
            }

            let key = match event.code {
                KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(PromptKey::Escape)
                }
                KeyCode::Up => Some(PromptKey::Up),
                KeyCode::Down => Some(PromptKey::Down),
                KeyCode::Enter => Some(PromptKey::Enter),
                KeyCode::Esc => Some(PromptKey::Escape),
                KeyCode::Backspace => Some(PromptKey::Backspace),
                KeyCode::Char(' ') => Some(PromptKey::Character(' ')),
                KeyCode::Char(ch) if !event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(PromptKey::Character(ch))
                }
                _ => None,
            };
            Ok(key)
        }

        /// Restores the terminal mode, returning any restoration error.
        pub fn restore(&mut self) -> io::Result<()> {
            if self.active {
                disable_raw_mode()?;
                self.active = false;
            }
            Ok(())
        }
    }

    impl Drop for LinuxKeyboard {
        fn drop(&mut self) {
            if self.active {
                let _ = disable_raw_mode();
            }
        }
    }
}

#[cfg(all(feature = "linux-keyboard", not(target_os = "linux")))]
compile_error!("the `linux-keyboard` feature is only available on Linux");

#[cfg(test)]
mod tests {
    use super::{ASCII_KEY_TABLE, AsciiKeyDecoder};
    use crate::PromptKey;

    #[test]
    fn ascii_table_maps_printable_characters_and_controls() {
        assert_eq!(
            ASCII_KEY_TABLE[b'a' as usize],
            Some(PromptKey::Character('a'))
        );
        assert_eq!(ASCII_KEY_TABLE[10], Some(PromptKey::Enter));
        assert_eq!(ASCII_KEY_TABLE[27], Some(PromptKey::Escape));
        assert_eq!(ASCII_KEY_TABLE[127], Some(PromptKey::Backspace));
        assert_eq!(ASCII_KEY_TABLE[1], None);
        assert_eq!(PromptKey::from_ascii(255), None);
    }

    #[test]
    fn decoder_reads_ascii_and_ansi_arrows_incrementally() {
        let mut decoder = AsciiKeyDecoder::new();
        assert_eq!(decoder.push(b'x'), Some(PromptKey::Character('x')));
        assert_eq!(decoder.push(0x1b), None);
        assert_eq!(decoder.push(b'['), None);
        assert_eq!(decoder.push(b'B'), Some(PromptKey::Down));
        assert_eq!(decoder.push(0x1b), None);
        assert_eq!(decoder.flush_escape(), Some(PromptKey::Escape));
    }
}
