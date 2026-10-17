use crate::{
    BOTTOM_LEFT_CORNER, BOTTOM_RIGHT_CORNER, DisplayMode, Draw, HORIZONTAL_LINE, LEFT_SPLITTER,
    Panel, RIGHT_SPLITTER, Status, TOP_LEFT_CORNER, TOP_RIGHT_CORNER, VERTICAL_LINE,
    cursor::set_cursor_pos, format_stack, print,
};

/// Character-cell renderer for server displays.
pub struct Serve {}

impl Default for Serve {
    fn default() -> Self {
        Self::new()
    }
}

impl Serve {
    /// Creates a server display renderer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Renders a server dashboard with a header and an automatically arranged panel grid.
    ///
    /// Screen dimensions are measured in character columns and rows. Screens
    /// smaller than 10 columns by 6 rows are left unchanged. A single panel
    /// occupies the available width; multiple panels use two columns, with an
    /// odd final panel spanning the full width. If the resulting panel rows
    /// would be shorter than three rows, the panels are not rendered.
    pub fn render_server_dashboard(
        screen_cols: usize,
        screen_rows: usize,
        system_title: &str,
        panels: &[Panel<'_>],
    ) {
        if screen_cols < 10 || screen_rows < 6 {
            return;
        }

        let header_height = 3;
        Self::draw_container(
            0,
            0,
            screen_cols,
            header_height,
            Some(" AMENTYS "),
            system_title,
        );

        if panels.is_empty() {
            return;
        }

        let body_y = header_height;
        let body_height = screen_rows.saturating_sub(header_height);

        if panels.len() == 1 {
            Self::render_styled_panel(0, body_y, screen_cols, body_height, &panels[0]);
            return;
        }

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

            let is_last_odd = (i == panels.len() - 1) && !panels.len().is_multiple_of(2);
            let width = if is_last_odd { screen_cols } else { col_width };

            Self::render_styled_panel(x, y, width, panel_height, panel);
        }
    }

    /// Draws a panel, including its title, status, content, and optional terminal drawer.
    fn render_styled_panel(x: usize, y: usize, width: usize, height: usize, panel: &Panel<'_>) {
        Self::draw_rect(x, y, width, height);

        let prefix = match panel.status {
            Status::Normal => " ",
            Status::Focused => "[>] ",
            Status::Alert => "[!] ",
        };

        // Draw the panel title on the left side of the top border.
        if width > 12 {
            let max_len = width - 10;
            set_cursor_pos(x + 2, y);
            for (count, ch) in prefix
                .chars()
                .chain(panel.title.chars())
                .chain(" ".chars())
                .enumerate()
            {
                if count >= max_len {
                    break;
                }
                print!("{}", ch);
            }

            // Show the terminal drawer state on the right side of the top border.
            let badge = if panel.term.is_open { "[▲]" } else { "[▼]" };
            Self::draw_text(x + width - 5, y, badge);
        }

        // Render the terminal drawer only when it is open and the panel is tall enough.
        if panel.term.is_open && height >= 6 {
            // Use roughly the top half of the panel for the drawer, at least three rows.
            let drop_height = (height / 2).max(3);
            let split_y = y + drop_height;

            // Show the command output and prompt in the drawer.
            let term_view = if panel.term.output.is_empty() {
                format_stack!("{} > {}_", panel.term.prompt, panel.term.input)
            } else {
                format_stack!(
                    "{}\n{} > {}_",
                    panel.term.output,
                    panel.term.prompt,
                    panel.term.input
                )
            };
            Self::draw_text_in_rect(x, y, width, drop_height + 1, term_view.as_str());

            // Separate the terminal drawer from the panel content.
            Self::draw_horizontal_split(x, split_y, width);

            // Show the panel content in the remaining space below the drawer.
            let remaining_height = height.saturating_sub(drop_height);
            Self::draw_text_in_rect(x, split_y, width, remaining_height, panel.content);
        } else {
            // With the drawer closed, the panel content uses the full interior.
            Self::draw_text_in_rect(x, y, width, height, panel.content);
        }
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

    fn draw_horizontal_split(x: usize, y: usize, width: usize) {
        if width < 2 {
            return;
        }
        Self::draw_point(x, y, LEFT_SPLITTER);
        if width > 2 {
            Self::draw_line(&DisplayMode::Horizontal, x + 1, y, width - 2);
        }
        Self::draw_point(x + width - 1, y, RIGHT_SPLITTER);
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

        if let Some(t) = title
            && width > 4
        {
            let max_title_len = width - 4;
            set_cursor_pos(x + 2, y);
            for (i, ch) in t.chars().enumerate() {
                if i >= max_title_len {
                    break;
                }
                print!("{}", ch);
            }
        }
        Self::draw_text_in_rect(x, y, width, height, text);
    }
}
