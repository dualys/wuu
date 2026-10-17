use core::sync::atomic::{AtomicUsize, Ordering};

static CURSOR_X: AtomicUsize = AtomicUsize::new(0);
static CURSOR_Y: AtomicUsize = AtomicUsize::new(0);

pub fn set_cursor_pos(x: usize, y: usize) {
    CURSOR_X.store(x, Ordering::SeqCst);
    CURSOR_Y.store(y, Ordering::SeqCst);
    // Déplace le curseur dans os_terminal (1-indexé)
    crate::print!("\x1b[{};{}H", y + 1, x + 1);
}

#[must_use]
pub fn get_cursor_pos() -> (usize, usize) {
    (
        CURSOR_X.load(Ordering::SeqCst),
        CURSOR_Y.load(Ordering::SeqCst),
    )
}

pub fn clear_screen() {
    crate::print!("\x1b[2J");
    set_cursor_pos(0, 0);
}
