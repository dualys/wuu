#![cfg_attr(not(test), no_std)]

mod drawing;
mod framebuffer;
mod model;
mod stack_string;

pub mod cursor;
pub mod message;
pub mod serve;
pub mod table;

pub use drawing::{
    BOTTOM_LEFT_CORNER, BOTTOM_RIGHT_CORNER, DisplayMode, Draw, HORIZONTAL_LINE, LEFT_SPLITTER,
    POINT_CHAR, RIGHT_SPLITTER, TOP_LEFT_CORNER, TOP_RIGHT_CORNER, VERTICAL_LINE,
};
pub use framebuffer::ScreenData;
pub use framebuffer::TERMINAL;
pub use message::{MessageLevel, print_message, write_message};
pub use model::{Context, DropDownTerm, Panel, Status};
pub use stack_string::StackString;
pub use table::Table;

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
