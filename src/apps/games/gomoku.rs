use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::UnifiedDisplay,
    ui::{Rect, Ui, UiEvents},
};
use embedded_graphics::pixelcolor::BinaryColor;
use rand::{RngExt, rngs::SmallRng};
use std::time::Instant;

const GOMOKU_SIZE: i32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GomokuView {
    Menu,
    Play,
}

pub struct GomokuState {
    board: [[i8; 10]; 10], // 0 = Empty, 1 = P1 (Solid), 2 = P2/AI (Hollow)
    cx: i32,
    cy: i32,
    is_pve: bool,
    difficulty: u8, // 0 = Easy, 1 = Hard
    turn: i8,       // 1 = P1, 2 = P2/AI
    winner: i8,     // 0 = Active, 1 = P1, 2 = P2/AI, 3 = Draw
    menu_select: usize,
    view: GomokuView,
    last_action: Instant,
    rng: SmallRng,
}

impl Default for GomokuState {
    fn default() -> Self {
        Self {
            board: [[0; 10]; 10],
            cx: GOMOKU_SIZE / 2,
            cy: GOMOKU_SIZE / 2,
            is_pve: true,
            difficulty: 1,
            turn: 1,
            winner: 0,
            menu_select: 0,
            view: GomokuView::Menu,
            last_action: Instant::now(),
            rng: rand::make_rng(),
        }
    }
}

impl GomokuState {
    fn reset_game(&mut self) {
        self.board = [[0; 10]; 10];
        self.cx = GOMOKU_SIZE / 2;
        self.cy = GOMOKU_SIZE / 2;
        self.turn = 1;
        self.winner = 0;
    }

    fn check_win(&self, x: i32, y: i32, role: i8) -> bool {
        let dx = [1, 0, 1, 1];
        let dy = [0, 1, 1, -1];

        for i in 0..4 {
            let mut count = 1;

            // Forward check
            let mut tx = x + dx[i];
            let mut ty = y + dy[i];
            while (0..GOMOKU_SIZE).contains(&tx)
                && (0..GOMOKU_SIZE).contains(&ty) // Fixed: safe boundary check
                && self.board[tx as usize][ty as usize] == role
            {
                count += 1;
                tx += dx[i];
                ty += dy[i];
            }

            // Backward check
            tx = x - dx[i];
            ty = y - dy[i];
            while (0..GOMOKU_SIZE).contains(&tx)
                && (0..GOMOKU_SIZE).contains(&ty)
                && self.board[tx as usize][ty as usize] == role
            {
                count += 1;
                tx -= dx[i];
                ty -= dy[i];
            }

            if count >= 5 {
                return true;
            }
        }
        false
    }

    fn evaluate_point(&self, x: i32, y: i32, role: i8) -> i32 {
        let dx = [1, 0, 1, 1];
        let dy = [0, 1, 1, -1];
        let mut total_weight = 0;

        for i in 0..4 {
            let mut count = 1;
            let mut open_ends = 0;

            let mut tx = x + dx[i];
            let mut ty = y + dy[i];
            while (0..GOMOKU_SIZE).contains(&tx)
                && (0..GOMOKU_SIZE).contains(&ty) // Fixed: safe boundary check
                && self.board[tx as usize][ty as usize] == role
            {
                count += 1;
                tx += dx[i];
                ty += dy[i];
            }
            if (0..GOMOKU_SIZE).contains(&tx)
                && (0..GOMOKU_SIZE).contains(&ty)
                && self.board[tx as usize][ty as usize] == 0
            {
                open_ends += 1;
            }

            tx = x - dx[i];
            ty = y - dy[i];
            while (0..GOMOKU_SIZE).contains(&tx)
                && (0..GOMOKU_SIZE).contains(&ty) // Fixed: safe boundary check
                && self.board[tx as usize][ty as usize] == role
            {
                count += 1;
                tx -= dx[i];
                ty -= dy[i];
            }
            if (0..GOMOKU_SIZE).contains(&tx)
                && (0..GOMOKU_SIZE).contains(&ty) // Fixed: safe boundary check
                && self.board[tx as usize][ty as usize] == 0
            {
                open_ends += 1;
            }

            if count >= 5 {
                total_weight += 5000;
            } else if count == 4 && open_ends == 2 {
                total_weight += 1200;
            } else if count == 4 && open_ends == 1 {
                total_weight += 800;
            } else if count == 3 && open_ends == 2 {
                total_weight += 400;
            } else if count == 3 && open_ends == 1 {
                total_weight += 100;
            } else if count == 2 && open_ends == 2 {
                total_weight += 30;
            }
        }
        total_weight
    }

    fn ai_move(&mut self) {
        let mut best_x = -1;
        let mut best_y = -1;
        let mut max_score = -1;

        if self.difficulty == 1 {
            // Hard Mode Heuristics
            for x in 0..GOMOKU_SIZE {
                for y in 0..GOMOKU_SIZE {
                    if self.board[x as usize][y as usize] == 0 {
                        let total_score = self.evaluate_point(x, y, 2)
                            + (self.evaluate_point(x, y, 1) as f32 * 1.2) as i32;
                        if total_score > max_score {
                            max_score = total_score;
                            best_x = x;
                            best_y = y;
                        }
                    }
                }
            }
        } else {
            // Easy Mode: Immediate offensive win scan
            for x in 0..GOMOKU_SIZE {
                for y in 0..GOMOKU_SIZE {
                    if self.board[x as usize][y as usize] == 0
                        && self.evaluate_point(x, y, 2) >= 1000
                    {
                        best_x = x;
                        best_y = y;
                        break;
                    }
                }
                if best_x != -1 {
                    break;
                }
            }

            // Immediate defensive block scan
            if best_x == -1 {
                for x in 0..GOMOKU_SIZE {
                    for y in 0..GOMOKU_SIZE {
                        if self.board[x as usize][y as usize] == 0
                            && self.evaluate_point(x, y, 1) >= 1000
                        {
                            best_x = x;
                            best_y = y;
                            break;
                        }
                    }
                    if best_x != -1 {
                        break;
                    }
                }
            }

            // Fallback to random legal placement coordinate
            if best_x == -1 {
                loop {
                    let rx = self.rng.random_range(0..GOMOKU_SIZE);
                    let ry = self.rng.random_range(0..GOMOKU_SIZE);
                    if self.board[rx as usize][ry as usize] == 0 {
                        best_x = rx;
                        best_y = ry;
                        break;
                    }
                }
            }
        }

        if best_x != -1 && best_y != -1 {
            self.board[best_x as usize][best_y as usize] = 2;
            self.cx = best_x;
            self.cy = best_y;
            if self.check_win(best_x, best_y, 2) {
                self.winner = 2;
            }
        }
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut GomokuState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    let now = Instant::now();
    let cooldown_elapsed = state.last_action.elapsed().as_millis() > 200;

    match state.view {
        GomokuView::Menu => {
            if cooldown_elapsed {
                // Vertical selector configurations (Y-Axis navigation)
                if ctx.input_manager.is_down(UiEvents::UP)
                    || ctx.input_manager.is_down(UiEvents::KEY_6)
                {
                    state.menu_select = if state.menu_select == 0 {
                        2
                    } else {
                        state.menu_select - 1
                    };
                    state.last_action = now;
                } else if ctx.input_manager.is_down(UiEvents::DOWN)
                    || ctx.input_manager.is_down(UiEvents::KEY_5)
                {
                    state.menu_select = (state.menu_select + 1) % 3;
                    state.last_action = now;
                }

                // Horizontal value flipping configurations (X-Axis navigation)
                if ctx.input_manager.is_down(UiEvents::LEFT)
                    || ctx.input_manager.is_down(UiEvents::KEY_4)
                {
                    if state.menu_select == 0 {
                        state.is_pve = !state.is_pve;
                        state.last_action = now;
                    } else if state.menu_select == 1 && state.is_pve {
                        state.difficulty = if state.difficulty == 0 { 1 } else { 0 };
                        state.last_action = now;
                    } else if state.menu_select == 2 {
                        return Some(App::games_menu());
                    }
                } else if ctx.input_manager.is_down(UiEvents::RIGHT)
                    || ctx.input_manager.is_down(UiEvents::KEY_7)
                {
                    if state.menu_select == 0 {
                        state.is_pve = !state.is_pve;
                        state.last_action = now;
                    } else if state.menu_select == 1 && state.is_pve {
                        state.difficulty = if state.difficulty == 0 { 1 } else { 0 };
                        state.last_action = now;
                    } else if state.menu_select == 2 {
                        state.reset_game();
                        state.view = GomokuView::Play;
                        state.last_action = now;
                    }
                }
            }

            if ctx.menu_events.contains(UiEvents::KEY_3)
                || ctx.menu_events.contains(UiEvents::CONFIRM) && state.menu_select == 2
            {
                state.reset_game();
                state.view = GomokuView::Play;
                state.last_action = now;
            }
        }
        GomokuView::Play => {
            if state.winner != 0 {
                if ctx.menu_events.contains(UiEvents::KEY_3)
                    || ctx.menu_events.contains(UiEvents::KEY_7)
                    || ctx.menu_events.contains(UiEvents::CONFIRM)
                {
                    state.view = GomokuView::Menu;
                    state.last_action = now;
                }
                return None;
            }

            // Handle standard player turn cursor updates
            if (state.turn == 1 || (state.turn == 2 && !state.is_pve)) && cooldown_elapsed {
                if (ctx.input_manager.is_down(UiEvents::UP)
                    || ctx.input_manager.is_down(UiEvents::KEY_6))
                    && state.cy > 0
                {
                    state.cy -= 1;
                    state.last_action = now;
                } else if (ctx.input_manager.is_down(UiEvents::DOWN)
                    || ctx.input_manager.is_down(UiEvents::KEY_5))
                    && state.cy < GOMOKU_SIZE - 1
                {
                    state.cy += 1;
                    state.last_action = now;
                }

                if (ctx.input_manager.is_down(UiEvents::LEFT)
                    || ctx.input_manager.is_down(UiEvents::KEY_4))
                    && state.cx > 0
                {
                    state.cx -= 1;
                    state.last_action = now;
                } else if (ctx.input_manager.is_down(UiEvents::RIGHT)
                    || ctx.input_manager.is_down(UiEvents::KEY_7))
                    && state.cx < GOMOKU_SIZE - 1
                {
                    state.cx += 1;
                    state.last_action = now;
                }
            }

            // Input handling triggers token placement
            let place_triggered = ctx.menu_events.contains(UiEvents::KEY_3)
                || ctx.menu_events.contains(UiEvents::CONFIRM);
            if place_triggered && state.board[state.cx as usize][state.cy as usize] == 0 {
                state.board[state.cx as usize][state.cy as usize] = state.turn;

                if state.check_win(state.cx, state.cy, state.turn) {
                    state.winner = state.turn;
                    return None;
                }

                // Check tie match mechanics
                let mut full = true;
                for i in 0..GOMOKU_SIZE {
                    for j in 0..GOMOKU_SIZE {
                        if state.board[i as usize][j as usize] == 0 {
                            full = false;
                        }
                    }
                }
                if full {
                    state.winner = 3;
                    return None;
                }

                state.turn = if state.turn == 1 { 2 } else { 1 };
                state.last_action = now;
            }

            // Handle asynchronous AI task execution steps
            if state.winner == 0 && state.turn == 2 && state.is_pve {
                state.ai_move();
                state.turn = 1;
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &GomokuState) {
    // Primary display handling (Render Game Grid Board Systems)
    let d1_3 = &mut ctx.display_1_3;
    let main_bounds = d1_3.rect();
    let mut ui_main = Ui::new(d1_3, ctx.font);

    // Secondary display handling (Information Statistics and Parameters)
    let d0_96 = &mut ctx.display_0_96;
    let info_bounds = d0_96.rect();
    let mut ui_info = Ui::new(d0_96, ctx.font);

    match state.view {
        GomokuView::Menu => {
            // Render information layout components inside auxiliary window context
            ui_info
                .label(Rect::new(0, 0, info_bounds.width, 11), "GOMOKU CONFIG")
                .center()
                .draw();
            ui_info.horizontal_divider(Rect::new(0, 11, info_bounds.width, 2));

            let m0 = if state.menu_select == 0 {
                "> Mode: "
            } else {
                "  Mode: "
            };
            let mode_str = if state.is_pve { "VS Comp" } else { "Dual P" };
            let mut buf0 = [0u8; 24];
            let str0 = msg_format(&mut buf0, m0, mode_str);
            ui_info
                .label(Rect::new(4, 16, info_bounds.width, 12), str0)
                .draw();

            let m1 = if state.menu_select == 1 {
                "> AI:   "
            } else {
                "  AI:   "
            };
            let diff_str = if state.is_pve {
                if state.difficulty == 0 {
                    "Easy"
                } else {
                    "Hard"
                }
            } else {
                "N/A"
            };
            let mut buf1 = [0u8; 24];
            let str1 = msg_format(&mut buf1, m1, diff_str);
            ui_info
                .label(Rect::new(4, 30, info_bounds.width, 12), str1)
                .draw();

            let m2 = if state.menu_select == 2 {
                "> [ START ]"
            } else {
                "  [ START ]"
            };
            ui_info
                .label(Rect::new(4, 46, info_bounds.width, 12), m2)
                .draw();

            // Render interactive title inside primary display viewport boundaries
            ui_main.label(main_bounds, "FIVE IN A ROW").center().draw();
        }
        GomokuView::Play => {
            // Game Matrix Constants setup variables
            let start_x = 36;
            let start_y = 4;
            let space = 6;

            // Draw line coordinate vectors processing grid patterns
            for i in 0..GOMOKU_SIZE {
                let offset = i * space;
                let length = (GOMOKU_SIZE - 1) * space + 1;
                ui_main.draw_filled_rect(
                    Rect::new(start_x, start_y + offset, length as u32, 1),
                    BinaryColor::On,
                );
                ui_main.draw_filled_rect(
                    Rect::new(start_x + offset, start_y, 1, length as u32),
                    BinaryColor::On,
                );
            }

            // Populate current token configurations across layout intersections
            for x in 0..GOMOKU_SIZE {
                for y in 0..GOMOKU_SIZE {
                    let cell = state.board[x as usize][y as usize];
                    let px = start_x + x * space;
                    let py = start_y + y * space;

                    if cell == 1 {
                        // Solid piece structure represents P1 configuration
                        ui_main.draw_filled_rect(Rect::new(px - 2, py - 2, 5, 5), BinaryColor::On);
                    } else if cell == 2 {
                        // Hollow piece structure represents P2/AI configurations
                        ui_main.draw_stroke_rect(
                            Rect::new(px - 2, py - 2, 5, 5),
                            BinaryColor::On,
                            1,
                        );
                    }
                }
            }

            // Draw targeting frame matrix over focused coordinates
            let cur_x = start_x + state.cx * space;
            let cur_y = start_y + state.cy * space;
            ui_main.draw_stroke_rect(Rect::new(cur_x - 3, cur_y - 3, 7, 7), BinaryColor::On, 1);

            // Output textual game analytics metrics into auxiliary dashboard
            ui_info
                .label(Rect::new(0, 0, info_bounds.width, 12), "MATCH INFO")
                .center()
                .draw();
            ui_info.horizontal_divider(Rect::new(0, 12, info_bounds.width, 2));

            let mode_lbl = if state.is_pve {
                "Mode: PvE"
            } else {
                "Mode: PvP"
            };
            ui_info
                .label(Rect::new(4, 18, info_bounds.width, 12), mode_lbl)
                .draw();

            if state.winner == 0 {
                let turn_str = if state.turn == 1 {
                    "Turn: P1 [X]"
                } else if state.is_pve {
                    "Turn: AI [O]"
                } else {
                    "Turn: P2 [O]"
                };
                ui_info
                    .label(Rect::new(4, 34, info_bounds.width, 12), turn_str)
                    .draw();
            } else {
                let win_lbl = match state.winner {
                    1 => "P1 WINS!",
                    2 => {
                        if state.is_pve {
                            "AI WINS!"
                        } else {
                            "P2 WINS!"
                        }
                    }
                    _ => "TIE MATCH",
                };
                ui_info
                    .label(Rect::new(4, 34, info_bounds.width, 12), win_lbl)
                    .draw();
                ui_info
                    .label(Rect::new(4, 48, info_bounds.width, 12), "[3] Reset")
                    .draw();
            }
        }
    }
}

fn msg_format<'a>(buf: &'a mut [u8], prefix: &str, suffix: &str) -> &'a str {
    let p_len = prefix.len();
    let s_len = suffix.len();
    if p_len + s_len > buf.len() {
        return "";
    }
    buf[..p_len].copy_from_slice(prefix.as_bytes());
    buf[p_len..p_len + s_len].copy_from_slice(suffix.as_bytes());
    core::str::from_utf8(&buf[..p_len + s_len]).unwrap_or("")
}
