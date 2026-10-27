use crate::PromptKey;

const fn ascii_key_table() -> [Option<PromptKey>; 128] {
    let mut table = [None; 128];
    let mut byte = b' ';
    while byte <= b'~' {
        table[byte as usize] = Some(PromptKey::Character(byte as char));
        byte += 1;
    }
    table[8] = Some(PromptKey::Backspace);
    table[9] = Some(PromptKey::Tab);
    table[10] = Some(PromptKey::Enter);
    table[13] = Some(PromptKey::Enter);
    table[27] = Some(PromptKey::Escape);
    table[127] = Some(PromptKey::Backspace);
    let mut control = 1;
    while control <= 26 {
        if control != 8 && control != 9 && control != 10 && control != 13 {
            table[control] = Some(PromptKey::Control((b'a' + control as u8 - 1) as char));
        }
        control += 1;
    }
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
/// [`flush_escape`](Self::flush_escape) after the host's escape timeout. CSI
/// and SS3 sequences cover navigation, editing, and page movement keys.
#[derive(Default)]
pub struct AsciiKeyDecoder {
    escape_state: u8,
    csi_params: [u8; 16],
    csi_len: u8,
}

impl AsciiKeyDecoder {
    /// Creates a decoder with no pending input.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            escape_state: 0,
            csi_params: [0; 16],
            csi_len: 0,
        }
    }

    /// Feeds one ASCII/ANSI byte and returns a decoded prompt key, if complete.
    ///
    /// Printable non-ASCII input should be decoded by the host and passed as
    /// `PromptKey::Character`.
    pub fn push(&mut self, byte: u8) -> Option<PromptKey> {
        match self.escape_state {
            0 if byte == 0x1b => {
                self.escape_state = 1;
                None
            }
            0 => PromptKey::from_ascii(byte),
            1 if byte == b'[' || byte == b'O' => {
                self.escape_state = if byte == b'[' { 2 } else { 3 };
                self.csi_len = 0;
                None
            }
            1 => {
                self.escape_state = 0;
                if byte.is_ascii_graphic() || byte == b' ' {
                    Some(PromptKey::Alt(byte as char))
                } else {
                    Some(PromptKey::Escape)
                }
            }
            2 if (0x40..=0x7e).contains(&byte) => {
                self.escape_state = 0;
                self.decode_csi(byte)
            }
            2 if (0x20..=0x3f).contains(&byte)
                && usize::from(self.csi_len) < self.csi_params.len() =>
            {
                self.csi_params[usize::from(self.csi_len)] = byte;
                self.csi_len += 1;
                None
            }
            3 => {
                self.escape_state = 0;
                decode_ss3(byte)
            }
            _ => {
                self.escape_state = 0;
                self.csi_len = 0;
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

    fn decode_csi(&self, final_byte: u8) -> Option<PromptKey> {
        match final_byte {
            b'A' => Some(PromptKey::Up),
            b'B' => Some(PromptKey::Down),
            b'C' => Some(PromptKey::Right),
            b'D' => Some(PromptKey::Left),
            b'H' => Some(PromptKey::Home),
            b'F' => Some(PromptKey::End),
            b'Z' => Some(PromptKey::Tab),
            b'~' => match first_parameter(&self.csi_params[..usize::from(self.csi_len)]) {
                1 | 7 => Some(PromptKey::Home),
                3 => Some(PromptKey::Delete),
                4 | 8 => Some(PromptKey::End),
                5 => Some(PromptKey::PageUp),
                6 => Some(PromptKey::PageDown),
                _ => None,
            },
            _ => None,
        }
    }
}

fn first_parameter(params: &[u8]) -> u16 {
    let mut value = 0u16;
    for byte in params.iter().copied() {
        if byte == b';' || byte == b':' {
            break;
        }
        if !byte.is_ascii_digit() {
            return 0;
        }
        value = value
            .saturating_mul(10)
            .saturating_add(u16::from(byte - b'0'));
    }
    value
}

fn decode_ss3(final_byte: u8) -> Option<PromptKey> {
    match final_byte {
        b'A' => Some(PromptKey::Up),
        b'B' => Some(PromptKey::Down),
        b'C' => Some(PromptKey::Right),
        b'D' => Some(PromptKey::Left),
        b'H' => Some(PromptKey::Home),
        b'F' => Some(PromptKey::End),
        _ => None,
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
                KeyCode::Char(ch) if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(PromptKey::Control(ch.to_ascii_lowercase()))
                }
                KeyCode::Char(ch) if event.modifiers.contains(KeyModifiers::ALT) => {
                    Some(PromptKey::Alt(ch))
                }
                KeyCode::Up => Some(PromptKey::Up),
                KeyCode::Down => Some(PromptKey::Down),
                KeyCode::Left => Some(PromptKey::Left),
                KeyCode::Right => Some(PromptKey::Right),
                KeyCode::Home => Some(PromptKey::Home),
                KeyCode::End => Some(PromptKey::End),
                KeyCode::Delete => Some(PromptKey::Delete),
                KeyCode::PageUp => Some(PromptKey::PageUp),
                KeyCode::PageDown => Some(PromptKey::PageDown),
                KeyCode::Tab | KeyCode::BackTab => Some(PromptKey::Tab),
                KeyCode::Enter => Some(PromptKey::Enter),
                KeyCode::Esc => Some(PromptKey::Escape),
                KeyCode::Backspace => Some(PromptKey::Backspace),
                KeyCode::Char(' ') => Some(PromptKey::Character(' ')),
                KeyCode::Char(ch) => Some(PromptKey::Character(ch)),
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
        assert_eq!(ASCII_KEY_TABLE[9], Some(PromptKey::Tab));
        assert_eq!(ASCII_KEY_TABLE[3], Some(PromptKey::Control('c')));
        assert_eq!(ASCII_KEY_TABLE[1], Some(PromptKey::Control('a')));
        assert_eq!(PromptKey::from_ascii(255), None);
    }

    #[test]
    fn decoder_reads_ascii_and_ansi_navigation_and_editing_keys() {
        let mut decoder = AsciiKeyDecoder::new();
        assert_eq!(decoder.push(b'x'), Some(PromptKey::Character('x')));
        assert_sequence(&mut decoder, b"\x1b[A", Some(PromptKey::Up));
        assert_sequence(&mut decoder, b"\x1b[B", Some(PromptKey::Down));
        assert_sequence(&mut decoder, b"\x1b[C", Some(PromptKey::Right));
        assert_sequence(&mut decoder, b"\x1b[D", Some(PromptKey::Left));
        assert_sequence(&mut decoder, b"\x1b[H", Some(PromptKey::Home));
        assert_sequence(&mut decoder, b"\x1b[F", Some(PromptKey::End));
        assert_sequence(&mut decoder, b"\x1b[3~", Some(PromptKey::Delete));
        assert_sequence(&mut decoder, b"\x1b[5~", Some(PromptKey::PageUp));
        assert_sequence(&mut decoder, b"\x1b[6~", Some(PromptKey::PageDown));
        assert_sequence(&mut decoder, b"\x1b[1;5D", Some(PromptKey::Left));
        assert_sequence(&mut decoder, b"\x1b[Z", Some(PromptKey::Tab));
        assert_sequence(&mut decoder, b"\x1bOD", Some(PromptKey::Left));
        assert_sequence(&mut decoder, b"\x1bOF", Some(PromptKey::End));
        assert_eq!(decoder.push(0x1b), None);
        assert_eq!(decoder.push(b'x'), Some(PromptKey::Alt('x')));
        assert_eq!(decoder.push(0x1b), None);
        assert_eq!(decoder.flush_escape(), Some(PromptKey::Escape));
    }

    fn assert_sequence(
        decoder: &mut AsciiKeyDecoder,
        sequence: &[u8],
        expected: Option<PromptKey>,
    ) {
        let mut decoded = None;
        for byte in sequence {
            if let Some(key) = decoder.push(*byte) {
                decoded = Some(key);
            }
        }
        assert_eq!(decoded, expected, "sequence: {sequence:?}");
    }
}
