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

pub struct Game2048State {
    board: [[u16; 4]; 4],
    score: u32,
    is_game_over: bool,
    last_action: Instant,
    rng: SmallRng,
}

impl Default for Game2048State {
    fn default() -> Self {
        let rng: SmallRng = rand::make_rng();

        let mut state = Self {
            board: [[0; 4]; 4],
            score: 0,
            is_game_over: false,
            last_action: Instant::now(),
            rng,
        };

        state.spawn_tile();
        state.spawn_tile();
        state
    }
}

impl Game2048State {
    fn spawn_tile(&mut self) {
        let mut count = 0;
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] == 0 {
                    count += 1;
                }
            }
        }
        if count == 0 {
            return;
        }

        let target = self.rng.random_range(0..count);
        let mut current = 0;
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] == 0 {
                    if current == target {
                        self.board[i][j] = if self.rng.random_range(0..10) == 0 {
                            4
                        } else {
                            2
                        };
                        return;
                    }
                    current += 1;
                }
            }
        }
    }

    fn check_game_over(&self) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] == 0 {
                    return false;
                }
                if i < 3 && self.board[i][j] == self.board[i + 1][j] {
                    return false;
                }
                if j < 3 && self.board[i][j] == self.board[i][j + 1] {
                    return false;
                }
            }
        }
        true
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut Game2048State) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC)
        || ctx.menu_events.contains(UiEvents::CONFIRM) && state.is_game_over
    {
        return Some(App::games_menu());
    }

    if state.is_game_over {
        return None;
    }

    let mut moved = false;

    // Control directional inputs with a 200ms debounce cooldown
    if state.last_action.elapsed().as_millis() > 200 {
        if ctx.input_manager.is_down(UiEvents::LEFT) {
            for i in 0..4 {
                let mut target = 0;
                for j in 0..4 {
                    if state.board[i][j] != 0 {
                        if target != j {
                            state.board[i][target] = state.board[i][j];
                            state.board[i][j] = 0;
                            moved = true;
                        }
                        target += 1;
                    }
                }
                for j in 0..3 {
                    if state.board[i][j] != 0 && state.board[i][j] == state.board[i][j + 1] {
                        state.board[i][j] *= 2;
                        state.score += state.board[i][j] as u32;
                        state.board[i][j + 1] = 0;
                        moved = true;
                    }
                }
                target = 0;
                for j in 0..4 {
                    if state.board[i][j] != 0 {
                        if target != j {
                            state.board[i][target] = state.board[i][j];
                            state.board[i][j] = 0;
                        }
                        target += 1;
                    }
                }
            }
            state.last_action = Instant::now();
        } else if ctx.input_manager.is_down(UiEvents::RIGHT) {
            for i in 0..4 {
                let mut target = 3;
                for j in (0..4).rev() {
                    if state.board[i][j] != 0 {
                        if target != j {
                            state.board[i][target] = state.board[i][j];
                            state.board[i][j] = 0;
                            moved = true;
                        }
                        target = target.saturating_sub(1);
                    }
                }
                for j in (1..4).rev() {
                    if state.board[i][j] != 0 && state.board[i][j] == state.board[i][j - 1] {
                        state.board[i][j] *= 2;
                        state.score += state.board[i][j] as u32;
                        state.board[i][j - 1] = 0;
                        moved = true;
                    }
                }
                target = 3;
                for j in (0..4).rev() {
                    if state.board[i][j] != 0 {
                        if target != j {
                            state.board[i][target] = state.board[i][j];
                            state.board[i][j] = 0;
                        }
                        target = target.saturating_sub(1);
                    }
                }
            }
            state.last_action = Instant::now();
        } else if ctx.input_manager.is_down(UiEvents::UP) {
            for j in 0..4 {
                let mut target = 0;
                for i in 0..4 {
                    if state.board[i][j] != 0 {
                        if target != i {
                            state.board[target][j] = state.board[i][j];
                            state.board[i][j] = 0;
                            moved = true;
                        }
                        target += 1;
                    }
                }
                for i in 0..3 {
                    if state.board[i][j] != 0 && state.board[i][j] == state.board[i + 1][j] {
                        state.board[i][j] *= 2;
                        state.score += state.board[i][j] as u32;
                        state.board[i + 1][j] = 0;
                        moved = true;
                    }
                }
                target = 0;
                for i in 0..4 {
                    if state.board[i][j] != 0 {
                        if target != i {
                            state.board[target][j] = state.board[i][j];
                            state.board[i][j] = 0;
                        }
                        target += 1;
                    }
                }
            }
            state.last_action = Instant::now();
        } else if ctx.input_manager.is_down(UiEvents::DOWN) {
            for j in 0..4 {
                let mut target = 3;
                for i in (0..4).rev() {
                    if state.board[i][j] != 0 {
                        if target != i {
                            state.board[target][j] = state.board[i][j];
                            state.board[i][j] = 0;
                            moved = true;
                        }
                        target = target.saturating_sub(1);
                    }
                }
                for i in (1..4).rev() {
                    if state.board[i][j] != 0 && state.board[i][j] == state.board[i - 1][j] {
                        state.board[i][j] *= 2;
                        state.score += state.board[i][j] as u32;
                        state.board[i - 1][j] = 0;
                        moved = true;
                    }
                }
                target = 3;
                for i in (0..4).rev() {
                    if state.board[i][j] != 0 {
                        if target != i {
                            state.board[target][j] = state.board[i][j];
                            state.board[i][j] = 0;
                        }
                        target = target.saturating_sub(1);
                    }
                }
            }
            state.last_action = Instant::now();
        }
    }

    if moved {
        state.spawn_tile();
        if state.check_game_over() {
            state.is_game_over = true;
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &Game2048State) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut rect_board = Rect::default();
    let mut rect_sidebar = Rect::default();

    // Divide layout into a 98px gameplay column and an interactive sidebar info block
    let root = FlexNode::new(FlexDirection::Row)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Column)
                .with_size(98, display_bounds.height)
                .assign_to(&mut rect_board),
        )
        .child(
            FlexNode::new(FlexDirection::Column)
                .with_flex(1)
                .assign_to(&mut rect_sidebar),
        );

    root.layout(display_bounds);

    // Draw Board Grid System
    let start_x = rect_board.x + 1;
    let start_y = rect_board.y + 2;

    for i in 0..=4 {
        // Horizontal Grid Dividers
        ui.draw_filled_rect(
            Rect::new(start_x, start_y + (i * 15), 96, 1),
            BinaryColor::On,
        );
        // Vertical Grid Dividers
        ui.draw_filled_rect(
            Rect::new(start_x + (i * 24), start_y, 1, 61),
            BinaryColor::On,
        );
    }

    // Render Active Numerical Elements via UI Centering Pipelines
    for i in 0..4 {
        for j in 0..4 {
            let val = state.board[i][j];
            if val != 0 {
                let cell_rect = Rect::new(
                    start_x + (j * 24) as i32 + 1,
                    start_y + (i * 15) as i32 + 1,
                    23,
                    14,
                );
                let mut val_bytes = [0u8; 8];
                let val_str = msg_format(&mut val_bytes, "", val as u32);
                ui.label(cell_rect, val_str).center().draw();
            }
        }
    }

    // Sidebar UI Content Layout
    let mut rect_title = Rect::default();
    let mut rect_divider = Rect::default();
    let mut rect_score_lbl = Rect::default();
    let mut rect_score_val = Rect::default();

    let sidebar_layout = FlexNode::new(FlexDirection::Column)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(rect_sidebar.width, 12)
                .assign_to(&mut rect_title),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(rect_sidebar.width, 2)
                .assign_to(&mut rect_divider),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(rect_sidebar.width, 12)
                .assign_to(&mut rect_score_lbl),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut rect_score_val),
        );

    sidebar_layout.layout(rect_sidebar);

    ui.label(rect_title, "2048").center().draw();
    ui.horizontal_divider(rect_divider);
    ui.label(rect_score_lbl, "SCORE").center().draw();

    let mut score_bytes = [0u8; 16];
    let score_str = msg_format(&mut score_bytes, "", state.score);
    ui.label(rect_score_val, score_str).center().draw();

    // Game Over Overlay Mask Modal Window
    if state.is_game_over {
        ui.draw_filled_rect(rect_board, BinaryColor::Off);
        ui.draw_stroke_rect(rect_board, BinaryColor::On, 1);

        let mut rect_line1 = Rect::default();
        let mut rect_line2 = Rect::default();

        let over_layout = FlexNode::new(FlexDirection::Column)
            .align_items(AlignItems::Stretch)
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_flex(1)
                    .assign_to(&mut rect_line1),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_flex(1)
                    .assign_to(&mut rect_line2),
            );

        over_layout.layout(rect_board);

        ui.label(rect_line1, "GAME OVER").center().draw();
        ui.label(rect_line2, "[Esc] Menu").center().draw();
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
        if idx == p_len {
            break;
        }
        idx -= 1;
    }
    let shift = idx + 1 - p_len;
    for i in p_len..buf.len() - shift {
        buf[i] = buf[i + shift];
    }
    core::str::from_utf8(&buf[..buf.len() - shift]).unwrap_or("")
}
