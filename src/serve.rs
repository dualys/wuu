use crate::{
    BOTTOM_LEFT_CORNER, BOTTOM_RIGHT_CORNER, DisplayMode, Draw, HORIZONTAL_LINE, Panel, Status,
    TOP_LEFT_CORNER, TOP_RIGHT_CORNER, VERTICAL_LINE, cursor::set_cursor_pos, print,
};

pub struct Serve {}

impl Default for Serve {
    fn default() -> Self {
        Self::new()
    }
}

impl Serve {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Agence automatiquement les panneaux en grille ("Bento Grid") pour le mode Serveur
    pub fn render_server_dashboard(
        screen_cols: usize,
        screen_rows: usize,
        system_title: &str,
        panels: &[Panel<'_>],
    ) {
        if screen_cols < 10 || screen_rows < 6 {
            return;
        }

        // 1. Bandeau supérieur du serveur (hauteur de 3 lignes)
        let header_height = 3;
        Self::draw_container(0, 0, screen_cols, header_height, Some(" AMENTYS "), system_title);

        if panels.is_empty() {
            return;
        }

        // 2. Calcul de l'espace restant pour la grille de panneaux
        let body_y = header_height;
        let body_height = screen_rows.saturating_sub(header_height);

        // Si un seul panneau, il prend tout l'écran
        if panels.len() == 1 {
            Self::render_styled_panel(0, body_y, screen_cols, body_height, &panels[0]);
            return;
        }

        // Sinon, découpage en grille de 2 colonnes
        let num_rows = panels.len().div_ceil(2);
        let panel_height = body_height / num_rows;
        let col_width = screen_cols / 2;

        if panel_height < 3 {
            return;
        }

        for (i, panel) in panels.iter().enumerate() {
            let row_idx = i / 2;
            let col_idx = i % 2;

            let x = col_idx * col_width;
            let y = body_y + (row_idx * panel_height);

            // Si c'est le dernier panneau et que le total est impair, il s'étend sur toute la largeur
            let is_last_odd = (i == panels.len() - 1) && (panels.len() % 2 != 0);
            let width = if is_last_odd {
                screen_cols
            } else {
                col_width
            };

            Self::render_styled_panel(x, y, width, panel_height, panel);
        }
    }

    /// Dessine un panneau en adaptant son titre selon son état (Normal, Focused, Alert)
    fn render_styled_panel(x: usize, y: usize, width: usize, height: usize, panel: &Panel<'_>) {
        Self::draw_rect(x, y, width, height);

        // Préfixe visuel selon l'état du panneau
        let prefix = match panel.status {
            Status::Normal => " ",
            Status::Focused => "[>] ",
            Status::Alert => "[!] ",
        };

        if width > 6 {
            let max_len = width - 4;
            set_cursor_pos(x + 2, y);
            let mut count = 0;

            for ch in prefix.chars().chain(panel.title.chars()).chain(" ".chars()) {
                if count >= max_len {
                    break;
                }
                print!("{}", ch);
                count += 1;
            }
        }

        Self::draw_text_in_rect(x, y, width, height, panel.content);
    }
}

impl Draw for Serve {
    fn draw_point(x: usize, y: usize, symbol: &str) {
        set_cursor_pos(x, y);
        print!("{}", symbol);
    }

    fn draw_line(display: &DisplayMode, x: usize, y: usize, len: usize) {
        match display {
            DisplayMode::Horizontal => {
                for i in 0..len {
                    Self::draw_point(x + i, y, HORIZONTAL_LINE);
                }
            }
            DisplayMode::Vertical => {
                for i in 0..len {
                    Self::draw_point(x, y + i, VERTICAL_LINE);
                }
            }
        }
    }

    fn draw_rect(x: usize, y: usize, width: usize, height: usize) {
        if width == 0 || height == 0 {
            return;
        }
        if width == 1 && height == 1 {
            Self::draw_point(x, y, TOP_LEFT_CORNER);
            return;
        }

        if width > 2 {
            Self::draw_line(&DisplayMode::Horizontal, x + 1, y, width - 2);
            if height > 1 {
                Self::draw_line(&DisplayMode::Horizontal, x + 1, y + height - 1, width - 2);
            }
        }

        if height > 2 {
            Self::draw_line(&DisplayMode::Vertical, x, y + 1, height - 2);
            if width > 1 {
                Self::draw_line(&DisplayMode::Vertical, x + width - 1, y + 1, height - 2);
            }
        }

        Self::draw_point(x, y, TOP_LEFT_CORNER);
        if width > 1 {
            Self::draw_point(x + width - 1, y, TOP_RIGHT_CORNER);
        }
        if height > 1 {
            Self::draw_point(x, y + height - 1, BOTTOM_LEFT_CORNER);
        }
        if width > 1 && height > 1 {
            Self::draw_point(x + width - 1, y + height - 1, BOTTOM_RIGHT_CORNER);
        }
    }

    fn fill_rect(x: usize, y: usize, width: usize, height: usize, symbol: &str) {
        for row in 0..height {
            for col in 0..width {
                Self::draw_point(x + col, y + row, symbol);
            }
        }
    }

    fn draw_square(x: usize, y: usize, size: usize) {
        Self::draw_rect(x, y, size, size);
    }

    fn draw_text(x: usize, y: usize, text: &str) {
        set_cursor_pos(x, y);
        print!("{}", text);
    }

    fn draw_text_in_rect(x: usize, y: usize, width: usize, height: usize, text: &str) {
        if width <= 2 || height <= 2 {
            return;
        }

        let inner_x = x + 1;
        let inner_y = y + 1;
        let max_cols = width - 2;
        let max_rows = height - 2;

        let mut col = 0;
        let mut row = 0;

        for ch in text.chars() {
            if ch == '\n' {
                row += 1;
                col = 0;
                if row >= max_rows {
                    break;
                }
                continue;
            }

            if col >= max_cols {
                row += 1;
                col = 0;
            }

            if row >= max_rows {
                break;
            }

            if col == 0 {
                set_cursor_pos(inner_x, inner_y + row);
            }

            print!("{}", ch);
            col += 1;
        }
    }

    fn draw_container(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        title: Option<&str>,
        text: &str,
    ) {
        Self::draw_rect(x, y, width, height);

        if let Some(t) = title {
            if width > 4 {
                let max_title_len = width - 4;
                set_cursor_pos(x + 2, y);
                for (i, ch) in t.chars().enumerate() {
                    if i >= max_title_len {
                        break;
                    }
                    print!("{}", ch);
                }
            }
        }

        Self::draw_text_in_rect(x, y, width, height, text);
    }
}