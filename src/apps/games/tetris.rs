use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::UnifiedDisplay,
    ui::{
        Rect, Ui, UiEvents,
        layout::{AlignItems, FlexDirection, FlexNode},
    },
};
use embedded_graphics::pixelcolor::BinaryColor;
use rand::{RngExt, rngs::SmallRng};
use std::time::Instant;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 16;

const TETROMINOES: [[[u8; 4]; 4]; 7] = [
    [[1, 1, 1, 1], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // I
    [[1, 1, 0, 0], [1, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // O
    [[0, 1, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // T
    [[0, 1, 1, 0], [1, 1, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // S
    [[1, 1, 0, 0], [0, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // Z
    [[1, 0, 0, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // J
    [[0, 0, 1, 0], [1, 1, 1, 0], [0, 0, 0, 0], [0, 0, 0, 0]], // L
];

pub struct TetrisState {
    board: [[bool; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: usize,
    piece_matrix: [[u8; 4]; 4],
    px: i32,
    py: i32,
    score: u32,
    is_game_over: bool,
    last_drop: Instant,
    rotate_locked: bool,
    rng: SmallRng,
}

impl Default for TetrisState {
    fn default() -> Self {
        let mut initial_state = Self {
            board: [[false; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: 0,
            piece_matrix: [[0; 4]; 4],
            px: 3,
            py: 0,
            score: 0,
            is_game_over: false,
            last_drop: Instant::now(),
            rotate_locked: false,
            rng: rand::make_rng(),
        };
        initial_state.spawn_piece();
        initial_state
    }
}

impl TetrisState {
    fn spawn_piece(&mut self) {
        // Generate a random piece index between 0 and 6 using SmallRng
        self.current_piece = self.rng.random_range(0..7);
        self.piece_matrix = TETROMINOES[self.current_piece];
        self.px = 3;
        self.py = 0;

        if self.check_collision(self.px, self.py, &self.piece_matrix) {
            self.is_game_over = true;
        }
    }

    fn check_collision(&self, next_x: i32, next_y: i32, matrix: &[[u8; 4]; 4]) -> bool {
        for (r, row) in matrix.iter().enumerate() {
            for (c, &val) in row.iter().enumerate() {
                if val != 0 {
                    let board_x = next_x + c as i32;
                    let board_y = next_y + r as i32;

                    if board_x < 0
                        || board_x >= BOARD_WIDTH as i32
                        || board_y >= BOARD_HEIGHT as i32
                    {
                        return true;
                    }
                    if board_y >= 0 && self.board[board_y as usize][board_x as usize] {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn get_ghost_y(&self) -> i32 {
        let mut ghost_y = self.py;
        while !self.check_collision(self.px, ghost_y + 1, &self.piece_matrix) {
            ghost_y += 1;
        }
        ghost_y
    }

    fn rotate_matrix(&mut self) {
        let mut next_matrix = [[0u8; 4]; 4];

        for (r, row) in self.piece_matrix.iter().enumerate() {
            for (c, &val) in row.iter().enumerate() {
                next_matrix[c][3 - r] = val;
            }
        }

        while !next_matrix[0].iter().any(|&v| v != 0) {
            next_matrix.rotate_left(1);
        }

        if !self.check_collision(self.px, self.py, &next_matrix) {
            self.piece_matrix = next_matrix;
        }
    }

    fn lock_and_clear(&mut self) {
        for r in 0..4 {
            for c in 0..4 {
                if self.piece_matrix[r][c] != 0 {
                    let board_y = self.py + r as i32;
                    let board_x = self.px + c as i32;
                    if board_y >= 0 && board_y < BOARD_HEIGHT as i32 {
                        self.board[board_y as usize][board_x as usize] = true;
                    }
                }
            }
        }

        let mut rows_cleared = 0;
        for r in 0..BOARD_HEIGHT {
            if self.board[r].iter().all(|&cell| cell) {
                rows_cleared += 1;
                for move_r in (1..=r).rev() {
                    self.board[move_r] = self.board[move_r - 1];
                }
                self.board[0] = [false; BOARD_WIDTH];
            }
        }

        self.score += match rows_cleared {
            1 => 40,
            2 => 100,
            3 => 300,
            4 => 1200,
            _ => 0,
        };

        self.spawn_piece();
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut TetrisState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    if state.is_game_over {
        if ctx.menu_events.contains(UiEvents::KEY_7) || ctx.menu_events.contains(UiEvents::CONFIRM)
        {
            *state = TetrisState::default();
        }
        return None;
    }

    let moving_left = ctx.input_manager.is_down(UiEvents::LEFT);
    let moving_right = ctx.input_manager.is_down(UiEvents::RIGHT);

    if moving_left && !state.check_collision(state.px - 1, state.py, &state.piece_matrix) {
        state.px -= 1;
    }
    if moving_right && !state.check_collision(state.px + 1, state.py, &state.piece_matrix) {
        state.px += 1;
    }

    let up_pressed =
        ctx.input_manager.is_down(UiEvents::UP) || ctx.input_manager.is_down(UiEvents::CONFIRM);
    if up_pressed {
        if !state.rotate_locked {
            state.rotate_matrix();
            state.rotate_locked = true;
        }
    } else {
        state.rotate_locked = false;
    }

    let drop_delay = if ctx.input_manager.is_down(UiEvents::DOWN) && !moving_left && !moving_right {
        40
    } else {
        700
    };

    if state.last_drop.elapsed().as_millis() >= drop_delay {
        if !state.check_collision(state.px, state.py + 1, &state.piece_matrix) {
            state.py += 1;
        } else {
            state.lock_and_clear();
        }
        state.last_drop = Instant::now();
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &TetrisState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut rect_game = Rect::default();
    let mut rect_sidebar = Rect::default();

    let root = FlexNode::new(FlexDirection::Row)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(2)
                .assign_to(&mut rect_game),
        )
        .child(
            FlexNode::new(FlexDirection::Column)
                .with_flex(1)
                .assign_to(&mut rect_sidebar),
        );

    root.layout(display_bounds);

    // Dynamic Calculation: Automatically derive the maximum block size that fits within the
    // allocated game area without overflowing (allocating 2 pixels for borders).
    let max_block_w = (rect_game.width as i32 - 2) / BOARD_WIDTH as i32;
    let max_block_h = (rect_game.height as i32 - 2) / BOARD_HEIGHT as i32;
    let block_size = std::cmp::min(max_block_w, max_block_h).max(1);

    let board_px_w = (BOARD_WIDTH * block_size as usize) as u32;
    let board_px_h = (BOARD_HEIGHT * block_size as usize) as u32;

    let center_x = rect_game.x + (rect_game.width as i32 - board_px_w as i32) / 2;
    let center_y = rect_game.y + (rect_game.height as i32 - board_px_h as i32) / 2;

    // Outer grid field limits frame
    ui.draw_stroke_rect(
        Rect::new(center_x - 1, center_y - 1, board_px_w + 2, board_px_h + 2),
        BinaryColor::On,
        1,
    );

    // Render accumulated landscape blocks
    for r in 0..BOARD_HEIGHT {
        for c in 0..BOARD_WIDTH {
            if state.board[r][c] {
                ui.draw_filled_rect(
                    Rect::new(
                        center_x + (c as i32 * block_size),
                        center_y + (r as i32 * block_size),
                        block_size as u32,
                        block_size as u32,
                    ),
                    BinaryColor::On,
                );
            }
        }
    }

    // Render preview ghost target drop projection coordinates
    let ghost_y = state.get_ghost_y();
    if ghost_y > state.py {
        for r in 0..4 {
            for c in 0..4 {
                if state.piece_matrix[r][c] != 0 {
                    let draw_y = center_y + ((ghost_y + r as i32) * block_size);
                    let draw_x = center_x + ((state.px + c as i32) * block_size);

                    if draw_y >= center_y {
                        ui.draw_stroke_rect(
                            Rect::new(draw_x, draw_y, block_size as u32, block_size as u32),
                            BinaryColor::On,
                            1,
                        );
                    }
                }
            }
        }
    }

    // Render currently descending active tetromino object
    for r in 0..4 {
        for c in 0..4 {
            if state.piece_matrix[r][c] != 0 {
                let draw_y = center_y + ((state.py + r as i32) * block_size);
                let draw_x = center_x + ((state.px + c as i32) * block_size);

                if draw_y >= center_y {
                    ui.draw_filled_rect(
                        Rect::new(draw_x, draw_y, block_size as u32, block_size as u32),
                        BinaryColor::On,
                    );
                }
            }
        }
    }

    // Sidebar status output metrics tracking
    let mut score_bytes = [0u8; 12];
    let score_str = msg_format(&mut score_bytes, "PTS:", state.score);

    let mut rect_lbl = Rect::new(rect_sidebar.x, rect_sidebar.y + 4, rect_sidebar.width, 12);
    ui.label(rect_lbl, "TETRIS").center().draw();
    rect_lbl.y += 16;
    ui.label(rect_lbl, score_str).center().draw();

    if state.is_game_over {
        ui.draw_filled_rect(rect_game, BinaryColor::Off);
        ui.draw_stroke_rect(rect_game, BinaryColor::On, 1);
        ui.label(rect_game, "G-OVER").center().draw();
    }
}

fn msg_format<'a>(buf: &'a mut [u8], prefix: &str, val: u32) -> &'a str {
    let p_len = prefix.len();
    buf[..p_len].copy_from_slice(prefix.as_bytes());
    let mut rem = val;
    let mut idx = buf.len() - 1;
    if rem == 0 {
        buf[p_len] = b'0';
        return core::str::from_utf8(&buf[..p_len + 1]).unwrap_or("");
    }
    while rem > 0 && idx >= p_len {
        buf[idx] = b'0' + (rem % 10) as u8;
        rem /= 10;
        idx -= 1;
    }
    let shift = idx + 1 - p_len;
    for i in p_len..buf.len() - shift {
        buf[i] = buf[i + shift];
    }
    core::str::from_utf8(&buf[..buf.len() - shift]).unwrap_or("")
}
