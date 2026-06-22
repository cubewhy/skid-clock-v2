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

const PADDLE_WIDTH: i32 = 24;
const PADDLE_HEIGHT: i32 = 3;
const PADDLE_Y: i32 = 58;

const BRICK_ROWS: usize = 3;
const BRICK_COLS: usize = 6;
const BRICK_WIDTH: i32 = 18;
const BRICK_HEIGHT: i32 = 4;
const BRICK_GAP_X: i32 = 3;
const BRICK_GAP_Y: i32 = 3;
const BRICK_OFFSET_X: i32 = 2;
const BRICK_OFFSET_Y: i32 = 13;
const MAX_DESIGNED_LEVELS: u32 = 3;

pub struct BrickState {
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    paddle_x: f32,
    grid: [[bool; BRICK_COLS]; BRICK_ROWS],
    score: u32,
    is_game_over: bool,
    is_game_win: bool,
    current_level: u32,
    last_tick: Instant,
    rng: SmallRng,
}

impl Default for BrickState {
    fn default() -> Self {
        let rng: SmallRng = rand::make_rng();

        let mut state = Self {
            ball_x: 64.0,
            ball_y: 45.0,
            ball_vx: 1.3,
            ball_vy: -1.3,
            paddle_x: (128 - PADDLE_WIDTH) as f32 / 2.0,
            grid: [[false; BRICK_COLS]; BRICK_ROWS],
            score: 0,
            is_game_over: false,
            is_game_win: false,
            current_level: 1,
            last_tick: Instant::now(),
            rng,
        };

        state.setup_level();
        state
    }
}

impl BrickState {
    fn setup_level(&mut self) {
        // 1. Reset ball location metrics to the center launch area
        self.ball_x = 64.0;
        self.ball_y = 45.0;

        // 2. Scale baseline dynamics speeds linearly per level step (+15% speed increase per level)
        let speed_multiplier = 1.0 + (self.current_level as f32 - 1.0) * 0.15;
        self.ball_vx = 1.3 * speed_multiplier;
        self.ball_vy = -1.3 * speed_multiplier;

        self.last_tick = Instant::now();

        // 3. Generate schematic matrix formation structural layout variants per level step
        for r in 0..BRICK_ROWS {
            for c in 0..BRICK_COLS {
                if self.current_level == 1 {
                    // Level 1: Traditional solid grid cluster formation
                    self.grid[r][c] = true;
                } else if self.current_level == 2 {
                    // Level 2: Checkerboard interlaced alternating configuration
                    self.grid[r][c] = (r + c) % 2 == 0;
                } else if self.current_level == 3 {
                    // Level 3: V-shaped inverted pyramid array distribution layout
                    self.grid[r][c] = c >= r && c < (BRICK_COLS - r);
                } else {
                    // Endless / Procedural Level: 60% probability target allocation chance
                    self.grid[r][c] = self.rng.random_range(0..10) < 6;
                }
            }
        }

        // Procedural generation safety fallback checkpoint constraint rule
        if self.current_level > MAX_DESIGNED_LEVELS {
            let mut has_any_brick = false;
            for r in 0..BRICK_ROWS {
                for c in 0..BRICK_COLS {
                    if self.grid[r][c] {
                        has_any_brick = true;
                    }
                }
            }
            // If random generation produces an empty map array, populate baseline index row 0 to protect execution flow
            if !has_any_brick {
                for c in 0..BRICK_COLS {
                    self.grid[0][c] = true;
                }
            }
        }
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut BrickState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC)
        || (state.is_game_over || state.is_game_win) && ctx.menu_events.contains(UiEvents::CONFIRM)
    {
        return Some(App::games_menu());
    }

    if state.is_game_over || state.is_game_win {
        if ctx.menu_events.contains(UiEvents::KEY_7) {
            *state = BrickState::default();
        }
        return None;
    }

    // Process kinematic steps using frame scaling delta updates (Targeting a 20ms baseline physics engine clock rate loop)
    let elapsed = state.last_tick.elapsed().as_millis();
    if elapsed < 20 {
        return None;
    }
    state.last_tick = Instant::now();

    let mut dt = (elapsed as f32) / 20.0;
    if dt > 3.0 {
        dt = 3.0;
    }

    // Calculate baseline paddle movement speed
    let mut paddle_speed = 3.5 * dt;

    // Boost movement speed when KEY_7 is actively held down
    if ctx.input_manager.is_down(UiEvents::KEY_7) {
        paddle_speed *= 1.5;
    }

    if ctx.input_manager.is_down(UiEvents::LEFT | UiEvents::KEY_4) {
        state.paddle_x -= paddle_speed;
    }
    if ctx.input_manager.is_down(UiEvents::RIGHT | UiEvents::KEY_5) {
        state.paddle_x += paddle_speed;
    }

    // Constrain structural boundaries properties
    let max_paddle_x = (128 - PADDLE_WIDTH) as f32;
    if state.paddle_x < 0.0 {
        state.paddle_x = 0.0;
    } else if state.paddle_x > max_paddle_x {
        state.paddle_x = max_paddle_x;
    }

    // Dynamic Ball vector updates execution paths
    state.ball_x += state.ball_vx * dt;
    state.ball_y += state.ball_vy * dt;

    // Outer wall boundary deflection bouncing parameters rules
    if state.ball_x <= 0.0 {
        state.ball_x = 0.0;
        state.ball_vx = -state.ball_vx;
    } else if state.ball_x >= 126.0 {
        state.ball_x = 126.0;
        state.ball_vx = -state.ball_vx;
    }

    // Upper boundary deflection check
    if state.ball_y <= 10.0 {
        state.ball_y = 10.0;
        state.ball_vy = -state.ball_vy;
    }

    // Lower pit void crash terminal defeat intercept condition rule
    if state.ball_y > 64.0 {
        state.is_game_over = true;
        return None;
    }

    // Paddle deflection tracking intersection box calculus
    if state.ball_y >= (PADDLE_Y - 2) as f32
        && state.ball_y <= (PADDLE_Y + 2) as f32
        && state.ball_x >= state.paddle_x - 1.0
        && state.ball_x <= state.paddle_x + PADDLE_WIDTH as f32 + 1.0
        && state.ball_vy > 0.0
    {
        state.ball_vy = -state.ball_vy;
        let hit_center_offset = (state.ball_x - state.paddle_x) / PADDLE_WIDTH as f32;
        // Map dynamic output reflection velocities scaled by hitting proximity from center axis
        state.ball_vx = 3.5 * (hit_center_offset - 0.5);
    }

    // Brick destruction logic loop validation sequence
    let mut has_bricks_left = false;
    for r in 0..BRICK_ROWS {
        for c in 0..BRICK_COLS {
            if state.grid[r][c] {
                has_bricks_left = true;

                let bx = (BRICK_OFFSET_X + c as i32 * (BRICK_WIDTH + BRICK_GAP_X)) as f32;
                let by = (BRICK_OFFSET_Y + r as i32 * (BRICK_HEIGHT + BRICK_GAP_Y)) as f32;

                // Evaluate standard AABB box footprint overlap boundaries intersections metrics
                if state.ball_x + 2.0 >= bx
                    && state.ball_x <= bx + BRICK_WIDTH as f32
                    && state.ball_y + 2.0 >= by
                    && state.ball_y <= by + BRICK_HEIGHT as f32
                {
                    state.grid[r][c] = false;
                    state.ball_vy = -state.ball_vy;
                    state.score += 1;

                    // Apply slight physical velocity compound acceleration rules step profiles increments (+1.5% speed modifier acceleration)
                    state.ball_vx *= 1.015;
                    state.ball_vy *= 1.015;
                    break;
                }
            }
        }
    }

    // Map clear detection logic pipeline execution criteria
    if !has_bricks_left {
        state.current_level += 1;
        state.setup_level();
        return None;
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &BrickState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut rect_lvl = Rect::default();
    let mut rect_score = Rect::default();
    let mut rect_divider = Rect::default();
    let mut rect_board = Rect::default();

    let root = FlexNode::new(FlexDirection::Column)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 11)
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_flex(1)
                        .assign_to(&mut rect_lvl),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_flex(1)
                        .assign_to(&mut rect_score),
                ),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 2)
                .assign_to(&mut rect_divider),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut rect_board),
        );

    root.layout(display_bounds);

    // Top Stats Line Rendering Processing
    let mut lvl_bytes = [0u8; 12];
    let lvl_str = msg_format(&mut lvl_bytes, "LVL:", state.current_level);
    ui.label(rect_lvl, lvl_str).center().draw();

    let mut score_bytes = [0u8; 16];
    let score_str = msg_format(&mut score_bytes, "SCORE:", state.score);
    ui.label(rect_score, score_str).center().draw();

    ui.horizontal_divider(rect_divider);

    // Populate active structural block bricks grid layouts elements
    for r in 0..BRICK_ROWS {
        for c in 0..BRICK_COLS {
            if state.grid[r][c] {
                let bx = BRICK_OFFSET_X + c as i32 * (BRICK_WIDTH + BRICK_GAP_X);
                let by = BRICK_OFFSET_Y + r as i32 * (BRICK_HEIGHT + BRICK_GAP_Y);
                ui.draw_filled_rect(
                    Rect::new(bx, by, BRICK_WIDTH as u32, BRICK_HEIGHT as u32),
                    BinaryColor::On,
                );
            }
        }
    }

    // Render active mobile platform paddle unit structures elements
    ui.draw_filled_rect(
        Rect::new(
            state.paddle_x as i32,
            PADDLE_Y,
            PADDLE_WIDTH as u32,
            PADDLE_HEIGHT as u32,
        ),
        BinaryColor::On,
    );

    // Render 2x2 tracking projectile physics particle object elements
    ui.draw_filled_rect(
        Rect::new(state.ball_x as i32, state.ball_y as i32, 2, 2),
        BinaryColor::On,
    );

    // === Game Over / Terminated Victory Overlays Display windows handlers ===
    if state.is_game_over || state.is_game_win {
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

        if state.is_game_win {
            ui.label(rect_line1, "YOU WIN!").center().draw();
        } else {
            ui.label(rect_line1, "GAME OVER").center().draw();
        }
        ui.label(rect_line2, "[7] Restart").center().draw();
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
