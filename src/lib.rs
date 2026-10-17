#![cfg_attr(not(test), no_std)]
pub mod cursor;
pub mod serve;

pub const HORIZONTAL_LINE: &str = "⎯";
pub const VERTICAL_LINE: &str = "|";

pub enum DisplayMode {
    Horizontal,
    Vertical,
}
pub trait Draw {
    fn draw_line(display: &DisplayMode, x: usize, y: usize, len: usize);
}
