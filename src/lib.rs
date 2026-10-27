#![cfg_attr(not(any(test, feature = "linux-keyboard")), no_std)]

mod drawing;
mod framebuffer;
mod model;
mod progress;
mod stack_string;
pub mod style;

pub mod console;
pub mod cursor;
pub mod diff;
pub mod keyboard;
pub mod message;
pub mod prompt;
pub mod serve;
pub mod table;

pub use console::{Arg, Command, Console, MAX_CONSOLE_ARGS, ParseError};
pub use diff::Diff;
pub use drawing::{
    BOTTOM_LEFT_CORNER, BOTTOM_RIGHT_CORNER, DisplayMode, Draw, HORIZONTAL_LINE, LEFT_SPLITTER,
    POINT_CHAR, RIGHT_SPLITTER, TOP_LEFT_CORNER, TOP_RIGHT_CORNER, VERTICAL_LINE,
};
pub use framebuffer::ScreenData;
pub use framebuffer::TERMINAL;
pub use keyboard::{ASCII_KEY_TABLE, AsciiKeyDecoder};
pub use message::{
    MessageLevel, ko, ko_custom, ok, ok_custom, print_message, print_styled_message, write_ko,
    write_ko_custom, write_message, write_ok, write_ok_custom, write_styled_message,
};
pub use model::{Context, DropDownTerm, Panel, Status};
pub use noun::Noun;
pub use progress::ProgressBar;
pub use prompt::{Prompt, PromptError, PromptKey, PromptResult, PromptState, TEXT_CAPACITY};
pub use stack_string::StackString;
pub use style::{Color, ColorMode};
pub use table::{Table, TableStyle};

/// High-level interface for clearing a display and rendering dashboards.
pub struct Wuu {
    context: Context,
}

impl Wuu {
    /// Creates an interface configured for the given display environment.
    #[must_use]
    pub const fn new(context: Context) -> Self {
        Self { context }
    }

    /// Changes the display environment used by this interface.
    pub fn set_context(&mut self, context: Context) {
        self.context = context;
    }

    /// Clears the terminal screen and returns the cursor to the top-left corner.
    pub fn clear(&self) {
        cursor::clear_screen();
    }

    /// Renders a dashboard for the active display environment.
    ///
    /// Dashboard rendering is currently implemented only for the server
    /// context. The cursor is hidden during rendering and its position and
    /// visibility are restored afterward.
    pub fn render_dashboard(&self, system_title: &str, panels: &[Panel<'_>]) {
        cursor::with_hidden_cursor(|| {
            if let Context::Server { cols, rows } = self.context {
                serve::Serve::render_server_dashboard(cols, rows, system_title, panels);
            }
        });
    }
}
