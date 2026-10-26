use os_terminal::{DrawTarget, Rgb, Terminal};
use spin::Mutex;

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
