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
use std::time::Instant;

const GRID_SIZE: usize = 8;
const CELL_SIZE: u32 = 5;
const START_Y: i32 = 14;
const START_X1: i32 = 4;
const START_X2: i32 = 68;
const JOY_DELAY_MS: u128 = 180; // Core delay mapping for discrete cursor steps

pub struct NavalBattleState {
    player_board: [[u8; GRID_SIZE]; GRID_SIZE],
    enemy_board: [[u8; GRID_SIZE]; GRID_SIZE],
    cursor_x: i8,
    cursor_y: i8,
    winner: u8, // 0 = Ongoing, 1 = Player Wins, 2 = AI Wins
    last_joy_action: Instant,
    rng_seed: u32,
}

impl Default for NavalBattleState {
    fn default() -> Self {
        let mut state = Self {
            player_board: [[0; GRID_SIZE]; GRID_SIZE],
            enemy_board: [[0; GRID_SIZE]; GRID_SIZE],
            cursor_x: 3,
            cursor_y: 3,
            winner: 0,
            last_joy_action: Instant::now(),
            rng_seed: 24680,
        };
        state.init_naval_game();
        state
    }
}

impl NavalBattleState {
    // Linear Congruential Generator matching original runtime execution rules
    fn pseudo_rand(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let rand_val = ((self.rng_seed / 65536) % 32768) as i32;
        min + (rand_val % (max - min))
    }

    fn place_random_ships(&mut self, is_player: bool) {
        let mut board = [[0u8; GRID_SIZE]; GRID_SIZE];
        let ship_sizes = [4, 3, 2, 2];

        for &size in &ship_sizes {
            let mut placed = false;
            while !placed {
                let dir = self.pseudo_rand(0, 2); // 0 = Horizontal, 1 = Vertical
                let x = self.pseudo_rand(0, 8);
                let y = self.pseudo_rand(0, 8);
                let mut valid = true;

                for i in 0..size {
                    let tx = x + if dir == 0 { i } else { 0 };
                    let ty = y + if dir == 1 { i } else { 0 };

                    if tx >= 8 || ty >= 8 || board[tx as usize][ty as usize] != 0 {
                        valid = false;
                        break;
                    }
                }

                if valid {
                    for i in 0..size {
                        let tx = x + if dir == 0 { i } else { 0 };
                        let ty = y + if dir == 1 { i } else { 0 };
                        board[tx as usize][ty as usize] = 1; // 1 = Intact Ship segment
                    }
                    placed = true;
                }
            }
        }

        if is_player {
            self.player_board = board;
        } else {
            self.enemy_board = board;
        }
    }

    fn check_naval_win(&self, board: &[[u8; GRID_SIZE]; GRID_SIZE]) -> bool {
        !board.iter().flatten().any(|&cell| cell == 1)
    }

    fn ai_move(&mut self) {
        loop {
            let rx = self.pseudo_rand(0, 8) as usize;
            let ry = self.pseudo_rand(0, 8) as usize;
            let val = self.player_board[rx][ry];

            if val == 0 || val == 1 {
                if val == 1 {
                    self.player_board[rx][ry] = 3; // 3 = Confirmed Hit
                } else {
                    self.player_board[rx][ry] = 2; // 2 = Confirmed Miss
                }

                if self.check_naval_win(&self.player_board) {
                    self.winner = 2; // AI Victory
                }
                break;
            }
        }
    }

    fn init_naval_game(&mut self) {
        self.winner = 0;
        self.cursor_x = 3;
        self.cursor_y = 3;
        self.place_random_ships(true);
        self.place_random_ships(false);
        self.last_joy_action = Instant::now();
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut NavalBattleState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    let clicked =
        ctx.input_manager.is_down(UiEvents::CONFIRM) || ctx.input_manager.is_down(UiEvents::KEY_3);

    if state.winner != 0 {
        if clicked {
            state.init_naval_game();
            return Some(App::games_menu());
        }
        return None;
    }

    // Process discrete navigation matrix steps safely using structured action delays
    let now = Instant::now();
    if now.duration_since(state.last_joy_action).as_millis() > JOY_DELAY_MS {
        let mut moved = false;

        if ctx.input_manager.is_down(UiEvents::UP | UiEvents::KEY_6) && state.cursor_y > 0 {
            state.cursor_y -= 1;
            moved = true;
        } else if ctx.input_manager.is_down(UiEvents::DOWN | UiEvents::KEY_5) && state.cursor_y < 7
        {
            state.cursor_y += 1;
            moved = true;
        }

        if ctx.input_manager.is_down(UiEvents::LEFT | UiEvents::KEY_4) && state.cursor_x > 0 {
            state.cursor_x -= 1;
            moved = true;
        } else if ctx.input_manager.is_down(UiEvents::RIGHT | UiEvents::KEY_7) && state.cursor_x < 7
        {
            state.cursor_x += 1;
            moved = true;
        }

        if moved {
            state.last_joy_action = now;
        }
    }

    // Fire command interceptor
    if clicked {
        let cx = state.cursor_x as usize;
        let cy = state.cursor_y as usize;
        let val = state.enemy_board[cx][cy];

        if val == 0 || val == 1 {
            if val == 1 {
                state.enemy_board[cx][cy] = 3;
            } else {
                state.enemy_board[cx][cy] = 2;
            }

            if state.check_naval_win(&state.enemy_board) {
                state.winner = 1; // Player Victory
                return None;
            }

            // Immediately shift turn control execution block to the automated AI opponent
            state.ai_move();
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &NavalBattleState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    if state.winner != 0 {
        let mut rect_title = Rect::default();
        let mut rect_hint = Rect::default();

        let root = FlexNode::new(FlexDirection::Column)
            .align_items(AlignItems::Center)
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 24)
                    .assign_to(&mut rect_title),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 16)
                    .assign_to(&mut rect_hint),
            );

        root.layout(display_bounds);

        if state.winner == 1 {
            ui.label(rect_title, "YOU WIN!").center().draw();
        } else {
            ui.label(rect_title, "AI WIN!").center().draw();
        }
        ui.label(rect_hint, "[PRESS CONFIRM] RETURN")
            .center()
            .draw();
        return;
    }

    // Top Dashboard Static Header Line Component
    ui.label(Rect::new(2, 0, 124, 10), "NAVAL BATTLE").draw();
    ui.draw_line((0, 10), (128, 10), BinaryColor::On);

    let board_box_dim = GRID_SIZE as u32 * CELL_SIZE + 2;

    // A. Render Player Fleet Board Matrix (Left Coordinates Space)
    ui.draw_stroke_rect(
        Rect::new(START_X1 - 1, START_Y - 1, board_box_dim, board_box_dim),
        BinaryColor::On,
        1,
    );
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let val = state.player_board[x][y];
            let cx = START_X1 + x as i32 * CELL_SIZE as i32;
            let cy = START_Y + y as i32 * CELL_SIZE as i32;

            if val == 1 {
                ui.draw_stroke_rect(
                    Rect::new(cx + 1, cy + 1, CELL_SIZE - 2, CELL_SIZE - 2),
                    BinaryColor::On,
                    1,
                );
            } else if val == 2 {
                ui.draw_filled_rect(Rect::new(cx + 2, cy + 2, 1, 1), BinaryColor::On);
            } else if val == 3 {
                ui.draw_filled_rect(
                    Rect::new(cx + 1, cy + 1, CELL_SIZE - 2, CELL_SIZE - 2),
                    BinaryColor::On,
                );
            }
        }
    }

    // B. Render Enemy Radar Board Matrix (Right Coordinates Space)
    ui.draw_stroke_rect(
        Rect::new(START_X2 - 1, START_Y - 1, board_box_dim, board_box_dim),
        BinaryColor::On,
        1,
    );
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let val = state.enemy_board[x][y];
            let cx = START_X2 + x as i32 * CELL_SIZE as i32;
            let cy = START_Y + y as i32 * CELL_SIZE as i32;

            if val == 2 {
                ui.draw_filled_rect(Rect::new(cx + 2, cy + 2, 1, 1), BinaryColor::On);
            } else if val == 3 {
                ui.draw_filled_rect(
                    Rect::new(cx + 1, cy + 1, CELL_SIZE - 2, CELL_SIZE - 2),
                    BinaryColor::On,
                );
            }
        }
    }

    // C. Render Active Targeting Target Scope Box Overlay onto the Radar Area
    let cur_px = START_X2 + state.cursor_x as i32 * CELL_SIZE as i32;
    let cur_py = START_Y + state.cursor_y as i32 * CELL_SIZE as i32;
    ui.draw_stroke_rect(
        Rect::new(cur_px, cur_py, CELL_SIZE, CELL_SIZE),
        BinaryColor::On,
        1,
    );

    // Footer Indicators Area
    ui.label(Rect::new(4, 56, 50, 8), "MY FLEET").draw();
    ui.label(Rect::new(68, 56, 50, 8), "EN RADAR").draw();
}
