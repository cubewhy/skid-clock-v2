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

/// Trait to manage access to different layout layers cleanly.
pub trait KeyboardLayer {
    fn get_key(&self, row: usize, col: usize) -> &'static str;
}

impl KeyboardLayer for KeyboardMode {
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

    pub fn handle_event(&mut self, events: UiEvents) {
        if events.contains(UiEvents::UP) {
            self.cursor_row = if self.cursor_row == 0 {
                3
            } else {
                self.cursor_row - 1
            };
        }
        if events.contains(UiEvents::DOWN) {
            self.cursor_row = (self.cursor_row + 1) % 4;
        }
        if events.contains(UiEvents::LEFT) {
            self.cursor_col = if self.cursor_col == 0 {
                9
            } else {
                self.cursor_col - 1
            };
        }
        if events.contains(UiEvents::RIGHT) {
            self.cursor_col = (self.cursor_col + 1) % 10;
        }
        if events.contains(UiEvents::KEY_3) {
            self.text.pop();
        }
        if events.contains(UiEvents::KEY_6) {
            self.mode.next_mode();
        }

        if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) {
            // Using the trait simplifies layout abstraction down to a single clean call
            let current_key = self.mode.get_key(self.cursor_row, self.cursor_col);

            match current_key {
                "abc" => self.mode = KeyboardMode::Lower,
                "ABC" => self.mode = KeyboardMode::Upper,
                "sym" | "SYM" => self.mode = KeyboardMode::Symbol,
                "Del" => {
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

// 4x10 Matrix Keyboards matching standard screen aspect ratios
const LAYOUT_LOWER: [[&str; 10]; 4] = [
    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    ["a", "s", "d", "f", "g", "h", "j", "k", "l", "ABC"],
    ["z", "x", "c", "v", "b", "n", "m", "sym", "Del", "OK"],
];

const LAYOUT_UPPER: [[&str; 10]; 4] = [
    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
    ["A", "S", "D", "F", "G", "H", "J", "K", "L", "abc"],
    ["Z", "X", "C", "V", "B", "N", "M", "SYM", "Del", "OK"],
];

const LAYOUT_SYM: [[&str; 10]; 4] = [
    ["!", "@", "#", "$", "%", "^", "&", "*", "(", ")"],
    ["-", "_", "=", "+", "[", "]", "{", "}", ";", ":"],
    ["'", "\"", ",", ".", "/", "<", ">", "?", "\\", "|"],
    ["~", "`", " ", " ", " ", " ", " ", "abc", "Del", "OK"],
];

impl<'a, D> Ui<'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    pub fn keyboard(&mut self, rect: Rect, state: &KeyboardState) {
        // 1. Draw Text Preview Box at the top
        let input_box_height = 14;
        let input_box = Rect::new(rect.x, rect.y, rect.width, input_box_height);
        self.draw_stroke_rect(input_box, BinaryColor::On, 1);

        let mut preview_text = state.text.clone();
        if preview_text.len() < state.max_length {
            preview_text.push('_'); // Render active text entry cursor
        }
        self.label(input_box.offset(3, 1), &preview_text).draw();

        // 2. Render Character Matrix Grid
        let grid_top = rect.y + input_box_height as i32 + 2;
        let grid_height = rect.height.saturating_sub(input_box_height + 2);

        for r in 0..4 {
            for c in 0..10 {
                // Cleaner layout resolving via the KeyboardLayer trait
                let key_text = state.mode.get_key(r, c);

                // Uniform pixel distribution matching fractional grid widths
                let cell_x = rect.x + (c as i32 * rect.width as i32) / 10;
                let next_x = rect.x + ((c + 1) as i32 * rect.width as i32) / 10;
                let cell_width = (next_x - cell_x) as u32;

                let cell_y = grid_top + (r as i32 * grid_height as i32) / 4;
                let next_y = grid_top + ((r + 1) as i32 * grid_height as i32) / 4;
                let cell_height = (next_y - cell_y) as u32;

                let cell_rect = Rect::new(cell_x, cell_y, cell_width, cell_height);
                let is_selected = (r == state.cursor_row) && (c == state.cursor_col);

                // Directly reuse your existing button widget
                self.button(cell_rect, key_text, is_selected);
            }
        }
    }
}
