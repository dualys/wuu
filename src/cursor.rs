use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

// Current cursor position.
static CURSOR_X: AtomicUsize = AtomicUsize::new(0);
static CURSOR_Y: AtomicUsize = AtomicUsize::new(0);

// Saved cursor position used by save and restore operations.
static SAVED_X: AtomicUsize = AtomicUsize::new(0);
static SAVED_Y: AtomicUsize = AtomicUsize::new(0);

// Cursor visibility and shape state.
static CURSOR_VISIBLE: AtomicBool = AtomicBool::new(true);
static CURSOR_SHAPE: AtomicU8 = AtomicU8::new(CursorShape::Block as u8);

/// Visual shape of the terminal cursor.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CursorShape {
    /// Block-shaped cursor.
    Block = 0,
    /// Underline-shaped cursor.
    Underline = 1,
    /// Vertical-bar-shaped cursor.
    Beam = 2,
}

/// Sets the cursor to an absolute, zero-based column and row.
pub fn set_cursor_pos(x: usize, y: usize) {
    CURSOR_X.store(x, Ordering::SeqCst);
    CURSOR_Y.store(y, Ordering::SeqCst);
    // The terminal escape sequence uses one-based coordinates.
    crate::print!("\x1b[{};{}H", y + 1, x + 1);
}

/// Returns the current cursor position as a zero-based `(column, row)` pair.
#[must_use]
pub fn get_cursor_pos() -> (usize, usize) {
    (
        CURSOR_X.load(Ordering::SeqCst),
        CURSOR_Y.load(Ordering::SeqCst),
    )
}

/// Moves the cursor to the top-left corner, at `(0, 0)`.
pub fn home() {
    set_cursor_pos(0, 0);
}

/// Moves the cursor by the signed column and row offsets.
///
/// The resulting position is clamped at zero and cannot become negative.
pub fn move_cursor_by(dx: isize, dy: isize) {
    let (x, y) = get_cursor_pos();
    let new_x = x.saturating_add_signed(dx);
    let new_y = y.saturating_add_signed(dy);
    set_cursor_pos(new_x, new_y);
}

/// Saves the current cursor position in memory.
pub fn save_cursor_pos() {
    let (x, y) = get_cursor_pos();
    SAVED_X.store(x, Ordering::SeqCst);
    SAVED_Y.store(y, Ordering::SeqCst);
}

/// Restores the cursor to the last position saved by [`save_cursor_pos`].
pub fn restore_cursor_pos() {
    let x = SAVED_X.load(Ordering::SeqCst);
    let y = SAVED_Y.load(Ordering::SeqCst);
    set_cursor_pos(x, y);
}

/// Hides the cursor on the terminal.
pub fn hide_cursor() {
    CURSOR_VISIBLE.store(false, Ordering::SeqCst);
    crate::print!("\x1b[?25l");
}

/// Shows the cursor on the terminal.
pub fn show_cursor() {
    CURSOR_VISIBLE.store(true, Ordering::SeqCst);
    crate::print!("\x1b[?25h");
}

/// Returns the cursor visibility state tracked by this module.
#[must_use]
pub fn is_cursor_visible() -> bool {
    CURSOR_VISIBLE.load(Ordering::SeqCst)
}

/// Sets the terminal cursor shape.
pub fn set_cursor_shape(shape: CursorShape) {
    CURSOR_SHAPE.store(shape as u8, Ordering::SeqCst);
    let code = match shape {
        CursorShape::Block => 2,
        CursorShape::Underline => 4,
        CursorShape::Beam => 6,
    };
    crate::print!("\x1b[{} q", code);
}

/// Runs a drawing operation with the cursor hidden, then restores its position
/// and original visibility.
///
/// If the cursor was already hidden, it remains hidden after the operation.
pub fn with_hidden_cursor<F: FnOnce()>(draw_fn: F) {
    let was_visible = is_cursor_visible();
    save_cursor_pos();
    hide_cursor();

    draw_fn();

    restore_cursor_pos();
    if was_visible {
        show_cursor();
    }
}

/// Clears the terminal screen and moves the cursor to `(0, 0)`.
pub fn clear_screen() {
    crate::print!("\x1b[2J");
    home();
}

#[cfg(test)]
mod tests {
    use super::{
        get_cursor_pos, hide_cursor, is_cursor_visible, move_cursor_by, restore_cursor_pos,
        save_cursor_pos, set_cursor_pos, show_cursor, with_hidden_cursor,
    };

    #[test]
    fn cursor_movement_and_hidden_guard_restore_state() {
        show_cursor();
        set_cursor_pos(3, 5);
        save_cursor_pos();

        move_cursor_by(-4, -8);
        assert_eq!(get_cursor_pos(), (0, 0));
        move_cursor_by(2, 3);
        assert_eq!(get_cursor_pos(), (2, 3));
        restore_cursor_pos();
        assert_eq!(get_cursor_pos(), (3, 5));

        with_hidden_cursor(|| {
            assert!(!is_cursor_visible());
            set_cursor_pos(20, 20);
        });
        assert!(is_cursor_visible());
        assert_eq!(get_cursor_pos(), (3, 5));

        hide_cursor();
        with_hidden_cursor(|| assert!(!is_cursor_visible()));
        assert!(!is_cursor_visible());
        show_cursor();
    }
}
