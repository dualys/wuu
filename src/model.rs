/// State and text for a window's contextual drop-down terminal.
#[derive(Copy, Clone)]
pub struct DropDownTerm<'a> {
    /// Whether the terminal drawer is open.
    pub is_open: bool,
    /// Context-specific prompt, such as a location or file name.
    pub prompt: &'a str,
    /// Command history or output for this window.
    pub output: &'a str,
    /// Command currently being entered.
    pub input: &'a str,
}

impl<'a> DropDownTerm<'a> {
    /// Creates a closed terminal drawer associated with the given prompt.
    #[must_use]
    pub const fn new(prompt: &'a str) -> Self {
        Self {
            is_open: false,
            prompt,
            output: "",
            input: "",
        }
    }

    /// Creates an open terminal drawer with the given output and current input.
    #[must_use]
    pub const fn open_with(prompt: &'a str, output: &'a str, input: &'a str) -> Self {
        Self {
            is_open: true,
            prompt,
            output,
            input,
        }
    }

    /// Toggles whether the terminal drawer is open.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }
}

/// Display environment and its dimensions.
#[derive(Copy, Clone)]
pub enum Context {
    /// Character-cell display, measured in columns and rows.
    Server {
        /// Display width in character columns.
        cols: usize,
        /// Display height in character rows.
        rows: usize,
    },
    /// Pixel-based console display.
    Console {
        /// Display width in pixels.
        width_px: usize,
        /// Display height in pixels.
        height_px: usize,
    },
    /// Pixel-based watch display, described by its pixel size.
    Watch {
        /// Display size in pixels.
        size_px: usize,
    },
}

/// Semantic panel state used to determine its visual style.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Status {
    /// Default panel appearance.
    Normal,
    /// Panel currently has focus.
    Focused,
    /// Panel is displaying an alert.
    Alert,
}

/// A logical window with content and an associated drop-down terminal.
#[derive(Copy, Clone)]
pub struct Panel<'a> {
    /// Panel title.
    pub title: &'a str,
    /// Panel body text.
    pub content: &'a str,
    /// Semantic state that controls the panel's visual style.
    pub status: Status,
    /// Contextual terminal state associated with this panel.
    pub term: DropDownTerm<'a>,
}

impl<'a> Panel<'a> {
    /// Creates a panel with its title, content, status, and terminal state.
    #[must_use]
    pub const fn new(
        title: &'a str,
        content: &'a str,
        status: Status,
        term: DropDownTerm<'a>,
    ) -> Self {
        Self {
            title,
            content,
            status,
            term,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DropDownTerm, Panel, Status};

    #[test]
    fn terminal_starts_closed_and_toggle_changes_state() {
        let mut terminal = DropDownTerm::new("src");

        assert!(!terminal.is_open);
        assert_eq!(terminal.prompt, "src");
        assert_eq!(terminal.output, "");
        assert_eq!(terminal.input, "");

        terminal.toggle();
        assert!(terminal.is_open);
        terminal.toggle();
        assert!(!terminal.is_open);
    }

    #[test]
    fn open_terminal_preserves_output_and_input() {
        let terminal = DropDownTerm::open_with("repo", "ready", "status");

        assert!(terminal.is_open);
        assert_eq!(terminal.prompt, "repo");
        assert_eq!(terminal.output, "ready");
        assert_eq!(terminal.input, "status");
    }

    #[test]
    fn panel_constructor_preserves_all_fields() {
        let terminal = DropDownTerm::new("repo");
        let panel = Panel::new("Git", "clean", Status::Focused, terminal);

        assert_eq!(panel.title, "Git");
        assert_eq!(panel.content, "clean");
        assert!(matches!(panel.status, Status::Focused));
        assert!(!panel.term.is_open);
    }
}
