#![cfg_attr(not(test), no_std)]

use core::fmt::{self, Write};
use os_terminal::{DrawTarget, Rgb, Terminal};
use spin::Mutex;

pub mod cursor;
pub mod serve;

use serve::Serve;

// Low-level drawing constants.
/// Character used to draw horizontal borders.
pub const HORIZONTAL_LINE: &str = "⎯";
/// Character used to draw vertical borders.
pub const VERTICAL_LINE: &str = "|";
/// Character used to draw a filled point.
pub const POINT_CHAR: &str = "█";
/// Character used for the top-left corner of a border.
pub const TOP_LEFT_CORNER: &str = "┌";
/// Character used for the top-right corner of a border.
pub const TOP_RIGHT_CORNER: &str = "┐";
/// Character used for the bottom-left corner of a border.
pub const BOTTOM_LEFT_CORNER: &str = "└";
/// Character used for the bottom-right corner of a border.
pub const BOTTOM_RIGHT_CORNER: &str = "┘";
/// Character used for the left junction of a horizontal divider.
pub const LEFT_SPLITTER: &str = "├";
/// Character used for the right junction of a horizontal divider.
pub const RIGHT_SPLITTER: &str = "┤";

/// Orientation used when drawing a line.
pub enum DisplayMode {
    /// Draw from left to right.
    Horizontal,
    /// Draw from top to bottom.
    Vertical,
}

/// Low-level drawing operations that use screen coordinates.
pub trait Draw {
    /// Draws a symbol at the specified column and row.
    fn draw_point(x: usize, y: usize, symbol: &str);

    /// Draws a horizontal or vertical line of the specified length.
    fn draw_line(display: &DisplayMode, x: usize, y: usize, len: usize);

    /// Draws the outline of a rectangle with the specified dimensions.
    fn draw_rect(x: usize, y: usize, width: usize, height: usize);

    /// Fills a rectangle with the specified symbol.
    fn fill_rect(x: usize, y: usize, width: usize, height: usize, symbol: &str);

    /// Draws the outline of a square with the specified side length.
    fn draw_square(x: usize, y: usize, size: usize);

    /// Draws text starting at the specified column and row.
    fn draw_text(x: usize, y: usize, text: &str);

    /// Draws text inside a rectangle, leaving its border intact.
    fn draw_text_in_rect(x: usize, y: usize, width: usize, height: usize, text: &str);

    /// Draws a horizontal divider of the specified width.
    fn draw_horizontal_split(x: usize, y: usize, width: usize);

    /// Draws a bordered container with an optional title and text content.
    fn draw_container(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        title: Option<&str>,
        text: &str,
    );
}

// =========================================================================
// Integrated drop-down terminal and high-level facade.
// =========================================================================

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
                Serve::render_server_dashboard(cols, rows, system_title, panels);
            }
        });
    }
}

// =========================================================================
// Stack formatting and framebuffer support.
// =========================================================================

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

/// Appends formatted text to the buffer, truncating output when it is full.
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

/// Framebuffer information used by the pixel drawing backend.
pub struct ScreenData {
    /// Pointer to the start of writable framebuffer memory.
    pub ptr: *mut u8,
    /// Framebuffer width in pixels.
    pub width: u64,
    /// Framebuffer height in pixels.
    pub height: u64,
    /// Byte distance between the start of consecutive framebuffer rows.
    pub pitch: u64,
}

unsafe impl Send for ScreenData {}
unsafe impl Sync for ScreenData {}

impl DrawTarget for ScreenData {
    fn size(&self) -> (usize, usize) {
        (
            usize::try_from(self.width).unwrap_or(0),
            usize::try_from(self.height).unwrap_or(0),
        )
    }

    #[inline]
    fn draw_pixel(&mut self, x: usize, y: usize, color: Rgb) {
        let screen_x = u64::try_from(x).unwrap_or(0);
        let screen_y = u64::try_from(y).unwrap_or(0);
        let offset = (screen_y * self.pitch) + (screen_x * 4);
        let offset_usize = usize::try_from(offset).unwrap_or(0);
        #[allow(clippy::cast_ptr_alignment)]
        unsafe {
            let pixel_ptr = self.ptr.add(offset_usize).cast::<u32>();
            let raw_color =
                (u32::from(color.0) << 16) | (u32::from(color.1) << 8) | u32::from(color.2);
            *pixel_ptr = raw_color;
        }
    }
}

/// Shared terminal instance used by the output macros.
///
/// Set this to `Some(terminal)` before using [`print!`] or [`println!`].
pub static TERMINAL: Mutex<Option<Terminal<ScreenData>>> = Mutex::new(None);

/// Writes formatted text to the configured terminal, if one is available.
///
/// Formatting errors are ignored. If no terminal is configured, this macro
/// produces no output.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        if let Some(term) = $crate::TERMINAL.lock().as_mut() {
            let _ = core::fmt::write(term, format_args!($($arg)*));
        }
    }};
}

/// Writes formatted text followed by a newline to the configured terminal.
///
/// Called without arguments, it writes only a newline. Formatting errors are
/// ignored, and no output is produced if no terminal is configured.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::print!("{}\n", format_args!($($arg)*)) };
}
