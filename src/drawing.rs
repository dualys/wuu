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
