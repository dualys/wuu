use crate::{DisplayMode, Draw, HORIZONTAL_LINE, VERTICAL_LINE, cursor::set_cursor_pos};

pub struct Serve {}

impl Default for Serve {
    fn default() -> Self {
        Self::new()
    }
}

impl Serve {
    pub fn new() -> Self {
        Self {}
    }
}

impl Draw for Serve {
    fn draw_line(display: &DisplayMode, x: usize, y: usize, len: usize) {
        match display {
            DisplayMode::Horizontal => {
                for i in x..len {
                    set_cursor_pos(x + i, y);
                }
            }
            DisplayMode::Vertical => {
                for i in y..len {
                    set_cursor_pos(x, y + i);
                }
            }
        }
    }
}
