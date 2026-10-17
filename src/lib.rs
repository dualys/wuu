#![cfg_attr(not(test), no_std)]

use os_terminal::{DrawTarget, Rgb, Terminal};
use spin::Mutex;

pub mod cursor;
pub mod serve;

use serve::Serve;

// --- Constantes de dessin bas niveau ---
pub const HORIZONTAL_LINE: &str = "⎯";
pub const VERTICAL_LINE: &str = "|";
pub const POINT_CHAR: &str = "█";
pub const TOP_LEFT_CORNER: &str = "┌";
pub const TOP_RIGHT_CORNER: &str = "┐";
pub const BOTTOM_LEFT_CORNER: &str = "└";
pub const BOTTOM_RIGHT_CORNER: &str = "┘";

pub enum DisplayMode {
    Horizontal,
    Vertical,
}

/// Moteur de rendu bas niveau (manipule des coordonnées X/Y)
pub trait Draw {
    fn draw_point(x: usize, y: usize, symbol: &str);
    fn draw_line(display: &DisplayMode, x: usize, y: usize, len: usize);
    fn draw_rect(x: usize, y: usize, width: usize, height: usize);
    fn fill_rect(x: usize, y: usize, width: usize, height: usize, symbol: &str);
    fn draw_square(x: usize, y: usize, size: usize);
    fn draw_text(x: usize, y: usize, text: &str);
    fn draw_text_in_rect(x: usize, y: usize, width: usize, height: usize, text: &str);
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
// LA FAÇADE HAUT NIVEAU (Aucune coordonnée X/Y exposée à l'utilisateur)
// =========================================================================

/// Définit l'environnement matériel sur lequel tourne l'interface
#[derive(Copy, Clone)]
pub enum Context {
    /// Mode Serveur (grille en colonnes et lignes de caractères)
    Server {
        cols: usize,
        rows: usize,
    },
    /// Prêt pour les futurs environnements (Console, TV, Montre, etc.)
    Console {
        width_px: usize,
        height_px: usize,
    },
    Watch {
        size_px: usize,
    },
}

/// État sémantique d'un contenant pour adapter son style visuel
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Status {
    Normal,
    Focused,
    Alert,
}

/// Un composant logique à afficher (indépendant de la position à l'écran)
#[derive(Copy, Clone)]
pub struct Panel<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub status: Status,
}

impl<'a> Panel<'a> {
    #[must_use]
    pub const fn new(title: &'a str, content: &'a str, status: Status) -> Self {
        Self {
            title,
            content,
            status,
        }
    }
}

/// La Façade principale utilisée par `ra` ou les applications
pub struct Wuu {
    context: Context,
}

impl Wuu {
    #[must_use]
    pub const fn new(context: Context) -> Self {
        Self { context }
    }

    /// Change dynamiquement de contexte (ex: passage d'un écran à un autre)
    pub fn set_context(&mut self, context: Context) {
        self.context = context;
    }

    /// Efface l'écran selon le contexte actif
    pub fn clear(&self) {
        match self.context {
            Context::Server { .. } | Context::Console { .. } | Context::Watch { .. } => {
                cursor::clear_screen();
            }
        }
    }

    /// Affiche une interface complète (Bandeau système + Grille automatique de panneaux)
    pub fn render_dashboard(&self, system_title: &str, panels: &[Panel<'_>]) {
        match self.context {
            Context::Server { cols, rows } => {
                Serve::render_server_dashboard(cols, rows, system_title, panels);
            }
            Context::Console { .. } | Context::Watch { .. } => {
                // Futurs moteurs de rendu (pixels / cartes)
            }
        }
    }
}

// =========================================================================
// GESTION DU FRAMEBUFFER ET MACROS D'AFFICHAGE
// =========================================================================

pub struct ScreenData {
    pub ptr: *mut u8,
    pub width: u64,
    pub height: u64,
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

pub static TERMINAL: Mutex<Option<Terminal<ScreenData>>> = Mutex::new(None);

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        if let Some(term) = $crate::TERMINAL.lock().as_mut() {
            let _ = core::fmt::write(term, format_args!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {$crate::print!("{}\n", format_args!($($arg)*)) };
}
