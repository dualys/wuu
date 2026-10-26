use core::cell::Cell;
use core::fmt::{self, Write};

use noun::Noun;

/// Maximum number of bytes stored by a text prompt.
pub const TEXT_CAPACITY: usize = 256;

/// Input event passed to a prompt by its host application.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PromptKey {
    /// A printable character.
    Character(char),
    /// Delete the previous character in a text prompt.
    Backspace,
    /// Move the active choice up.
    Up,
    /// Move the active choice down.
    Down,
    /// Toggle the current choice or confirmation.
    Toggle,
    /// Accept the current value.
    Enter,
    /// Cancel the prompt.
    Escape,
}

/// Current lifecycle state of a prompt.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PromptState {
    /// Waiting for input.
    Pending,
    /// The value was accepted.
    Accepted,
    /// The prompt was cancelled.
    Cancelled,
}

/// Value returned by an accepted prompt.
pub enum PromptResult<'a> {
    /// Entered text.
    Text(&'a str),
    /// Confirmation value.
    Confirm(bool),
    /// Selected index for a single-choice prompt.
    Selected(usize),
    /// Selection flags corresponding to a multi-choice prompt's options.
    Multiple(&'a [bool]),
    /// Selected noun identifier.
    Noun(&'a Noun),
}

/// Error creating a prompt with invalid choices.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PromptError {
    /// Select and noun prompts require at least one option.
    NoChoices,
    /// A multi-select flag is required for each option.
    SelectionCountMismatch {
        /// Number of provided options.
        options: usize,
        /// Number of provided selection flags.
        selections: usize,
    },
}

#[derive(Copy, Clone)]
enum PromptKind<'a> {
    Text,
    Confirm,
    Select(&'a [&'a str]),
    MultiSelect(&'a [&'a str]),
    Nouns(&'a [Noun]),
}

/// Event-driven, allocation-free interactive prompt.
///
/// The host application supplies input with [`handle_key`](Self::handle_key)
/// and calls [`print`](Self::print) or [`write_to`](Self::write_to) to redraw.
/// This lets a VCS integrate its own keyboard/event loop with `wuu`.
pub struct Prompt<'a> {
    message: &'a str,
    kind: PromptKind<'a>,
    cursor: usize,
    text: [u8; TEXT_CAPACITY],
    text_len: usize,
    selections: Option<&'a mut [bool]>,
    noun_prefix_len: usize,
    state: PromptState,
    rendered_lines: Cell<usize>,
}

impl<'a> Prompt<'a> {
    /// Creates a text-entry prompt.
    #[must_use]
    pub const fn text(message: &'a str) -> Self {
        Self::new(message, PromptKind::Text)
    }

    /// Creates a yes/no confirmation prompt, initially set to no.
    #[must_use]
    pub const fn confirm(message: &'a str) -> Self {
        Self::new(message, PromptKind::Confirm)
    }

    /// Creates a single-choice prompt.
    ///
    /// Returns [`PromptError::NoChoices`] when `options` is empty.
    pub fn select(message: &'a str, options: &'a [&'a str]) -> Result<Self, PromptError> {
        if options.is_empty() {
            return Err(PromptError::NoChoices);
        }
        Ok(Self::new(message, PromptKind::Select(options)))
    }

    /// Creates a multi-choice prompt using caller-owned selection flags.
    ///
    /// Each flag corresponds to the option at the same index. The caller
    /// provides the storage to keep this API allocation-free. All flags start
    /// false when the prompt is created.
    pub fn multi_select(
        message: &'a str,
        options: &'a [&'a str],
        selections: &'a mut [bool],
    ) -> Result<Self, PromptError> {
        if options.is_empty() {
            return Err(PromptError::NoChoices);
        }
        if options.len() != selections.len() {
            return Err(PromptError::SelectionCountMismatch {
                options: options.len(),
                selections: selections.len(),
            });
        }
        selections.fill(false);
        let mut prompt = Self::new(message, PromptKind::MultiSelect(options));
        prompt.selections = Some(selections);
        Ok(prompt)
    }

    /// Creates a noun selector displaying the leading hash bytes as hex.
    ///
    /// By default, eight hash bytes (16 hex characters) are shown. Use
    /// [`noun_prefix_len`](Self::noun_prefix_len) to change this.
    pub fn noun_select(message: &'a str, nouns: &'a [Noun]) -> Result<Self, PromptError> {
        if nouns.is_empty() {
            return Err(PromptError::NoChoices);
        }
        Ok(Self::new(message, PromptKind::Nouns(nouns)))
    }

    const fn new(message: &'a str, kind: PromptKind<'a>) -> Self {
        Self {
            message,
            kind,
            cursor: 0,
            text: [0; TEXT_CAPACITY],
            text_len: 0,
            selections: None,
            noun_prefix_len: 8,
            state: PromptState::Pending,
            rendered_lines: Cell::new(0),
        }
    }

    /// Sets the number of leading noun hash bytes shown (0 to 32).
    #[must_use]
    pub fn noun_prefix_len(mut self, bytes: usize) -> Self {
        self.noun_prefix_len = bytes.min(32);
        self
    }

    /// Applies a key event. Once accepted or cancelled, further input is ignored.
    pub fn handle_key(&mut self, key: PromptKey) {
        if self.state != PromptState::Pending {
            return;
        }

        if key == PromptKey::Escape {
            self.state = PromptState::Cancelled;
            return;
        }

        match self.kind {
            PromptKind::Text => self.handle_text_key(key),
            PromptKind::Confirm => self.handle_confirm_key(key),
            PromptKind::Select(options) => {
                self.handle_choice_key(key, options.len(), false);
            }
            PromptKind::MultiSelect(options) => {
                self.handle_choice_key(key, options.len(), true);
            }
            PromptKind::Nouns(nouns) => {
                self.handle_choice_key(key, nouns.len(), false);
            }
        }
    }

    /// Returns the prompt lifecycle state.
    #[must_use]
    pub const fn state(&self) -> PromptState {
        self.state
    }

    /// Returns the accepted value, or `None` while pending or after cancellation.
    #[must_use]
    pub fn result(&self) -> Option<PromptResult<'_>> {
        if self.state != PromptState::Accepted {
            return None;
        }
        match self.kind {
            PromptKind::Text => Some(PromptResult::Text(
                core::str::from_utf8(&self.text[..self.text_len]).unwrap_or(""),
            )),
            PromptKind::Confirm => Some(PromptResult::Confirm(self.cursor == 1)),
            PromptKind::Select(_) => Some(PromptResult::Selected(self.cursor)),
            PromptKind::MultiSelect(_) => self.selections.as_deref().map(PromptResult::Multiple),
            PromptKind::Nouns(nouns) => Some(PromptResult::Noun(&nouns[self.cursor])),
        }
    }

    /// Writes the prompt, clearing the previous rendering on subsequent calls.
    pub fn write_to<W: Write>(&self, output: &mut W) -> fmt::Result {
        let previous_lines = self.rendered_lines.get();
        if previous_lines > 0 {
            write!(output, "\x1b[{previous_lines}A\r")?;
            for line in 0..previous_lines {
                output.write_str("\x1b[2K")?;
                if line + 1 < previous_lines {
                    output.write_str("\x1b[1B\r")?;
                }
            }
            if previous_lines > 1 {
                write!(output, "\x1b[{}A\r", previous_lines - 1)?;
            }
        }

        let lines = self.render_content(output)?;
        self.rendered_lines.set(lines);
        Ok(())
    }

    /// Prints or refreshes the prompt on the configured terminal.
    pub fn print(&self) {
        if let Some(term) = crate::TERMINAL.lock().as_mut() {
            let _ = self.write_to(term);
        }
    }

    fn handle_text_key(&mut self, key: PromptKey) {
        match key {
            PromptKey::Character(ch) if !ch.is_control() => {
                let mut encoded = [0; 4];
                let bytes = ch.encode_utf8(&mut encoded).as_bytes();
                if self.text_len + bytes.len() <= TEXT_CAPACITY {
                    self.text[self.text_len..self.text_len + bytes.len()].copy_from_slice(bytes);
                    self.text_len += bytes.len();
                }
            }
            PromptKey::Backspace if self.text_len > 0 => {
                self.text_len -= 1;
                while self.text_len > 0 && self.text[self.text_len] & 0b1100_0000 == 0b1000_0000 {
                    self.text_len -= 1;
                }
            }
            PromptKey::Enter => self.state = PromptState::Accepted,
            _ => {}
        }
    }

    fn handle_confirm_key(&mut self, key: PromptKey) {
        match key {
            PromptKey::Toggle => self.cursor = 1 - self.cursor,
            PromptKey::Character('y' | 'Y') => self.cursor = 1,
            PromptKey::Character('n' | 'N') => self.cursor = 0,
            PromptKey::Enter => self.state = PromptState::Accepted,
            _ => {}
        }
    }

    fn handle_choice_key(&mut self, key: PromptKey, count: usize, multi: bool) {
        match key {
            PromptKey::Up => self.cursor = self.cursor.saturating_sub(1),
            PromptKey::Down => self.cursor = (self.cursor + 1).min(count - 1),
            PromptKey::Toggle if multi => self.toggle_selection(),
            PromptKey::Character(' ') if multi => self.toggle_selection(),
            PromptKey::Enter => self.state = PromptState::Accepted,
            _ => {}
        }
    }

    fn toggle_selection(&mut self) {
        if let Some(selections) = self.selections.as_deref_mut()
            && let Some(selected) = selections.get_mut(self.cursor)
        {
            *selected = !*selected;
        }
    }

    fn render_content<W: Write>(&self, output: &mut W) -> Result<usize, fmt::Error> {
        output.write_str(self.message)?;
        match self.kind {
            PromptKind::Text => {
                output.write_str(": ")?;
                output
                    .write_str(core::str::from_utf8(&self.text[..self.text_len]).unwrap_or(""))?;
                output.write_char('_')?;
                output.write_char('\n')?;
                Ok(1)
            }
            PromptKind::Confirm => {
                output.write_str(" [y/N] ")?;
                output.write_str(if self.cursor == 1 { "yes" } else { "no" })?;
                output.write_char('\n')?;
                Ok(1)
            }
            PromptKind::Select(options) | PromptKind::MultiSelect(options) => {
                output.write_char('\n')?;
                for (index, option) in options.iter().enumerate() {
                    output.write_str(if index == self.cursor { "> " } else { "  " })?;
                    if matches!(self.kind, PromptKind::MultiSelect(_)) {
                        let checked = self
                            .selections
                            .as_deref()
                            .and_then(|selections| selections.get(index))
                            .copied()
                            .unwrap_or(false);
                        output.write_str(if checked { "[x] " } else { "[ ] " })?;
                    }
                    output.write_str(option)?;
                    output.write_char('\n')?;
                }
                Ok(options.len() + 1)
            }
            PromptKind::Nouns(nouns) => {
                output.write_char('\n')?;
                for (index, noun) in nouns.iter().enumerate() {
                    output.write_str(if index == self.cursor { "> " } else { "  " })?;
                    write_noun_prefix(output, noun, self.noun_prefix_len)?;
                    output.write_char('\n')?;
                }
                Ok(nouns.len() + 1)
            }
        }
    }
}

fn write_noun_prefix<W: Write>(output: &mut W, noun: &Noun, bytes: usize) -> fmt::Result {
    for byte in noun.as_bytes().iter().take(bytes) {
        write!(output, "{byte:02x}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Prompt, PromptKey, PromptResult, PromptState};
    use noun::Noun;

    #[test]
    fn text_prompt_accepts_unicode_and_backspace() {
        let mut prompt = Prompt::text("Commit message");
        prompt.handle_key(PromptKey::Character('é'));
        prompt.handle_key(PromptKey::Character('x'));
        prompt.handle_key(PromptKey::Backspace);
        prompt.handle_key(PromptKey::Enter);

        assert!(matches!(prompt.result(), Some(PromptResult::Text("é"))));
    }

    #[test]
    fn select_returns_selected_option_index() {
        let options = ["main", "feature"];
        let mut prompt = Prompt::select("Branch", &options).unwrap();
        prompt.handle_key(PromptKey::Down);
        prompt.handle_key(PromptKey::Enter);

        assert!(matches!(prompt.result(), Some(PromptResult::Selected(1))));
    }

    #[test]
    fn multi_select_updates_caller_storage() {
        let options = ["stage", "commit"];
        let mut selections = [false; 2];
        let mut prompt = Prompt::multi_select("Actions", &options, &mut selections).unwrap();
        prompt.handle_key(PromptKey::Toggle);
        prompt.handle_key(PromptKey::Down);
        prompt.handle_key(PromptKey::Toggle);
        prompt.handle_key(PromptKey::Enter);

        assert!(matches!(
            prompt.result(),
            Some(PromptResult::Multiple([true, true]))
        ));
    }

    #[test]
    fn noun_prompt_returns_noun_and_renders_hash_prefix() {
        let noun = Noun::of(b"file contents");
        let nouns = [noun];
        let mut prompt = Prompt::noun_select("Choose object", &nouns).unwrap();
        prompt.handle_key(PromptKey::Enter);

        assert!(matches!(
            prompt.result(),
            Some(PromptResult::Noun(selected)) if selected == &nouns[0]
        ));

        let mut display = String::new();
        prompt.write_to(&mut display).unwrap();
        let prefix = nouns[0]
            .as_bytes()
            .iter()
            .take(8)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert!(display.contains(&prefix));
    }

    #[test]
    fn escape_cancels_prompt() {
        let mut prompt = Prompt::confirm("Continue?");
        prompt.handle_key(PromptKey::Escape);
        prompt.handle_key(PromptKey::Enter);

        assert_eq!(prompt.state(), PromptState::Cancelled);
        assert!(prompt.result().is_none());
    }

    #[test]
    fn redraw_clears_the_full_previous_prompt() {
        let options = ["main", "feature"];
        let mut prompt = Prompt::select("Branch", &options).unwrap();
        let mut display = String::new();

        prompt.write_to(&mut display).unwrap();
        prompt.handle_key(PromptKey::Down);
        prompt.write_to(&mut display).unwrap();

        let redraw = &display[display.find("\x1b[3A\r").unwrap()..];
        assert_eq!(redraw.matches("\x1b[2K").count(), 3);
        assert!(redraw.starts_with("\x1b[3A\r"));
        assert!(redraw.contains("> feature"));
    }
}
