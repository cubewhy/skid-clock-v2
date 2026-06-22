use crate::ui::ctx::{Ui, UiEvents};
use crate::ui::layout::Rect;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardMode {
    Lower,
    Upper,
    Symbol,
}

impl KeyboardMode {
    /// Cycles to the next keyboard layer mode.
    pub fn next_mode(&mut self) {
        *self = match self {
            KeyboardMode::Lower => KeyboardMode::Upper,
            KeyboardMode::Upper => KeyboardMode::Symbol,
            KeyboardMode::Symbol => KeyboardMode::Lower,
        };
    }
}

/// Trait to manage access to dynamic layout layers cleanly.
pub trait KeyboardLayer {
    fn get_row_count(&self) -> usize;
    fn get_row_len(&self, row: usize) -> usize;
    fn get_key(&self, row: usize, col: usize) -> &'static str;
}

impl KeyboardLayer for KeyboardMode {
    #[inline]
    fn get_row_count(&self) -> usize {
        match self {
            KeyboardMode::Lower => LAYOUT_LOWER.len(),
            KeyboardMode::Upper => LAYOUT_UPPER.len(),
            KeyboardMode::Symbol => LAYOUT_SYM.len(),
        }
    }

    #[inline]
    fn get_row_len(&self, row: usize) -> usize {
        match self {
            KeyboardMode::Lower => LAYOUT_LOWER[row].len(),
            KeyboardMode::Upper => LAYOUT_UPPER[row].len(),
            KeyboardMode::Symbol => LAYOUT_SYM[row].len(),
        }
    }

    #[inline]
    fn get_key(&self, row: usize, col: usize) -> &'static str {
        match self {
            KeyboardMode::Lower => LAYOUT_LOWER[row][col],
            KeyboardMode::Upper => LAYOUT_UPPER[row][col],
            KeyboardMode::Symbol => LAYOUT_SYM[row][col],
        }
    }
}

pub struct KeyboardState {
    pub text: String,
    pub max_length: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub mode: KeyboardMode,
    pub confirmed: bool,
}

impl KeyboardState {
    pub fn new(max_length: usize) -> Self {
        Self {
            text: String::new(),
            max_length,
            cursor_row: 0,
            cursor_col: 0,
            mode: KeyboardMode::Lower,
            confirmed: false,
        }
    }

    /// Clamps current cursors safely within the boundary limits of the active mode layer
    fn clamp_cursors(&mut self) {
        let row_count = self.mode.get_row_count();
        if self.cursor_row >= row_count {
            self.cursor_row = row_count - 1;
        }
        let row_len = self.mode.get_row_len(self.cursor_row);
        if self.cursor_col >= row_len {
            self.cursor_col = row_len - 1;
        }
    }

    pub fn handle_event(&mut self, events: UiEvents) {
        let row_count = self.mode.get_row_count();
        let current_row_len = self.mode.get_row_len(self.cursor_row);

        if events.contains(UiEvents::UP) {
            self.cursor_row = if self.cursor_row == 0 {
                row_count - 1
            } else {
                self.cursor_row - 1
            };
            let new_row_len = self.mode.get_row_len(self.cursor_row);
            if self.cursor_col >= new_row_len {
                self.cursor_col = new_row_len - 1;
            }
        }
        if events.contains(UiEvents::DOWN) {
            self.cursor_row = (self.cursor_row + 1) % row_count;
            let new_row_len = self.mode.get_row_len(self.cursor_row);
            if self.cursor_col >= new_row_len {
                self.cursor_col = new_row_len - 1;
            }
        }
        if events.contains(UiEvents::LEFT) {
            self.cursor_col = if self.cursor_col == 0 {
                current_row_len - 1
            } else {
                self.cursor_col - 1
            };
        }
        if events.contains(UiEvents::RIGHT) {
            self.cursor_col = (self.cursor_col + 1) % current_row_len;
        }
        if events.contains(UiEvents::KEY_3) {
            self.text.pop();
        }
        if events.contains(UiEvents::KEY_6) {
            self.mode.next_mode();
            self.clamp_cursors();
        }

        if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) {
            let current_key = self.mode.get_key(self.cursor_row, self.cursor_col);

            match current_key {
                "abc" => {
                    self.mode = KeyboardMode::Lower;
                    self.clamp_cursors();
                }
                "ABC" => {
                    self.mode = KeyboardMode::Upper;
                    self.clamp_cursors();
                }
                "sym" | "SYM" => {
                    self.mode = KeyboardMode::Symbol;
                    self.clamp_cursors();
                }
                "<-" => {
                    self.text.pop();
                }
                "OK" => {
                    self.confirmed = true;
                }
                " " => {
                    if self.text.len() < self.max_length {
                        self.text.push(' ');
                    }
                }
                character => {
                    if self.text.len() < self.max_length {
                        self.text.push_str(character);
                    }
                }
            }
        }
    }
}

// Optimized layout arrangements:
// Moving numbers/extra symbols off letter layers maximizes functional key sizing on smaller screens.
const LAYOUT_LOWER: &[&[&str]] = &[
    &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"], // 10 columns
    &["a", "s", "d", "f", "g", "h", "j", "k", "l"],      // 9 columns
    &["z", "x", "c", "v", "b", "n", "m"],                // 7 columns
    &["ABC", "sym", "<-", "OK"], // 4 columns (Gives "sym", "<-", "OK" tons of space!)
];

const LAYOUT_UPPER: &[&[&str]] = &[
    &["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
    &["A", "S", "D", "F", "G", "H", "J", "K", "L"],
    &["Z", "X", "C", "V", "B", "N", "M"],
    &["abc", "SYM", "<-", "OK"],
];

const LAYOUT_SYM: &[&[&str]] = &[
    &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    &["!", "@", "#", "$", "%", "^", "&", "*", "(", ")"],
    &["-", "_", "=", "+", "[", "]", "{", "}", ";", ":"],
    &["'", "\"", ",", ".", "/", "<", ">", "?", "\\", "|"],
    &["abc", " ", "<-", "OK"], // 4 columns for clean spacing
];

impl<'a, D> Ui<'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    pub fn keyboard(&mut self, rect: Rect, state: &KeyboardState, title: &str) {
        // 1. Draw Title Text Area at the top
        let title_height = 12;
        let title_rect = Rect::new(rect.x, rect.y, rect.width, title_height);
        self.label(title_rect.offset(3, 0), title).draw();

        // 2. Draw Text Preview Box below the title area
        let input_box_height = 14;
        let input_box_y = rect.y + title_height as i32 + 2;
        let input_box = Rect::new(rect.x, input_box_y, rect.width, input_box_height);
        self.draw_stroke_rect(input_box, BinaryColor::On, 1);

        let mut preview_text = state.text.clone();
        if preview_text.len() < state.max_length {
            preview_text.push('_'); // Render active text entry cursor
        }
        self.label(input_box.offset(3, 1), &preview_text).draw();

        // 3. Render Optimized Layout Grid
        let grid_top = input_box_y + input_box_height as i32 + 2;
        let grid_height = rect
            .height
            .saturating_sub(title_height + 2 + input_box_height + 2);

        let row_count = state.mode.get_row_count();

        for r in 0..row_count {
            let row_len = state.mode.get_row_len(r);

            let cell_y = grid_top + (r as i32 * grid_height as i32) / row_count as i32;
            let next_y = grid_top + ((r + 1) as i32 * grid_height as i32) / row_count as i32;
            let cell_height = (next_y - cell_y) as u32;

            for c in 0..row_len {
                let key_text = state.mode.get_key(r, c);

                // Dynamically computes pixel distribution using row-specific item count definitions
                let cell_x = rect.x + (c as i32 * rect.width as i32) / row_len as i32;
                let next_x = rect.x + ((c + 1) as i32 * rect.width as i32) / row_len as i32;
                let cell_width = (next_x - cell_x) as u32;

                let cell_rect = Rect::new(cell_x, cell_y, cell_width, cell_height);
                let is_selected = (r == state.cursor_row) && (c == state.cursor_col);

                // Directly reuse your existing button widget
                self.button(cell_rect, key_text, is_selected);
            }
        }
    }
}
