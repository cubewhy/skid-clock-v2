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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Menu,
    Playing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    PvE, // Player vs AI
    PvP, // Player vs Player
}

pub struct PongState {
    phase: GamePhase,
    game_mode: GameMode,
    player_y: f32,
    enemy_y: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32, // Pixels per second
    ball_vy: f32, // Pixels per second
    player_score: u32,
    enemy_score: u32,
    last_update: Instant,
    accumulator: f32, // Time accumulator for fixed timestep physics
}

impl Default for PongState {
    fn default() -> Self {
        Self {
            phase: GamePhase::Menu,
            game_mode: GameMode::PvE,
            player_y: 16.0,
            enemy_y: 16.0,
            ball_x: 64.0,
            ball_y: 24.0,
            // 60.0 pixels/sec at 60Hz fixed timestep = exactly 1.0 pixel per frame movement
            ball_vx: 60.0,
            ball_vy: 30.0,
            player_score: 0,
            enemy_score: 0,
            last_update: Instant::now(),
            accumulator: 0.0,
        }
    }
}

impl PongState {
    fn reset_ball(&mut self, serve_to_player: bool) {
        self.ball_x = 64.0;
        self.ball_y = 24.0;
        self.ball_vx = if serve_to_player { -60.0 } else { 60.0 };
        self.ball_vy = 30.0;
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut PongState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    // Calculate Delta Time
    let now = Instant::now();
    let mut dt = now.duration_since(state.last_update).as_secs_f32();
    state.last_update = now;

    // Cap delta time to prevent massive jumps during lag spikes
    if dt > 0.1 {
        dt = 0.1;
    }

    match state.phase {
        GamePhase::Menu => {
            if ctx.input_manager.is_down(UiEvents::UP) {
                state.game_mode = GameMode::PvE;
                state.phase = GamePhase::Playing;
                state.last_update = Instant::now();
                state.accumulator = 0.0;
            } else if ctx.input_manager.is_down(UiEvents::DOWN) {
                state.game_mode = GameMode::PvP;
                state.phase = GamePhase::Playing;
                state.last_update = Instant::now();
                state.accumulator = 0.0;
            }
        }
        GamePhase::Playing => {
            // Target 60Hz for internal physics steps (approx 16.67ms per tick)
            const TIME_STEP: f32 = 1.0 / 60.0;
            state.accumulator += dt;

            // Consume time slices inside fixed intervals to stabilize physics updates
            while state.accumulator >= TIME_STEP {
                let area_height = 50.0;
                let paddle_height = 10.0;
                let max_y = area_height - paddle_height;
                let paddle_speed = 100.0;

                // Player 1 input tracking
                if ctx.input_manager.is_down(UiEvents::UP) {
                    state.player_y = (state.player_y - paddle_speed * TIME_STEP).max(0.0);
                }
                if ctx.input_manager.is_down(UiEvents::DOWN) {
                    state.player_y = (state.player_y + paddle_speed * TIME_STEP).min(max_y);
                }

                // Enemy / Player 2 input routing
                match state.game_mode {
                    GameMode::PvE => {
                        let enemy_center = state.enemy_y + paddle_height / 2.0;
                        let ai_speed = 50.0;
                        if state.ball_y > enemy_center && state.enemy_y < max_y {
                            state.enemy_y = (state.enemy_y + ai_speed * TIME_STEP).min(max_y);
                        } else if state.ball_y < enemy_center && state.enemy_y > 0.0 {
                            state.enemy_y = (state.enemy_y - ai_speed * TIME_STEP).max(0.0);
                        }
                    }
                    GameMode::PvP => {
                        if ctx.input_manager.is_down(UiEvents::KEY_5) {
                            state.enemy_y = (state.enemy_y - paddle_speed * TIME_STEP).max(0.0);
                        }
                        if ctx.input_manager.is_down(UiEvents::KEY_6) {
                            state.enemy_y = (state.enemy_y + paddle_speed * TIME_STEP).min(max_y);
                        }
                    }
                }

                // Stable physics position step updates
                state.ball_x += state.ball_vx * TIME_STEP;
                state.ball_y += state.ball_vy * TIME_STEP;

                // Top / Bottom wall bounds rebound collisions
                if state.ball_y <= 0.0 {
                    state.ball_y = 0.0;
                    state.ball_vy = -state.ball_vy;
                } else if state.ball_y >= area_height - 2.0 {
                    state.ball_y = area_height - 2.0;
                    state.ball_vy = -state.ball_vy;
                }

                // Player Left Paddle Collision Bounds
                if state.ball_vx < 0.0
                    && state.ball_x <= 4.0
                    && state.ball_x >= 1.0
                    && state.ball_y >= state.player_y
                    && state.ball_y <= state.player_y + paddle_height
                {
                    state.ball_vx = -state.ball_vx * 1.05;
                    state.ball_vy += (state.ball_y - (state.player_y + paddle_height / 2.0)) * 7.5;
                    state.ball_x = 4.1;
                }

                // Enemy/P2 Right Paddle Collision Bounds
                if state.ball_vx > 0.0
                    && state.ball_x >= 122.0
                    && state.ball_x <= 125.0
                    && state.ball_y >= state.enemy_y
                    && state.ball_y <= state.enemy_y + paddle_height
                {
                    state.ball_vx = -state.ball_vx * 1.05;
                    state.ball_vy += (state.ball_y - (state.enemy_y + paddle_height / 2.0)) * 7.5;
                    state.ball_x = 121.9;
                }

                // Point Scoring Cycle Triggers
                if state.ball_x < 0.0 {
                    state.enemy_score += 1;
                    state.reset_ball(false);
                } else if state.ball_x > 128.0 {
                    state.player_score += 1;
                    state.reset_ball(true);
                }

                state.accumulator -= TIME_STEP;
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &PongState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    match state.phase {
        GamePhase::Menu => {
            let mut rect_title = Rect::default();
            let mut rect_opt1 = Rect::default();
            let mut rect_opt2 = Rect::default();

            let root = FlexNode::new(FlexDirection::Column)
                .align_items(AlignItems::Center)
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(display_bounds.width, 16)
                        .assign_to(&mut rect_title),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(display_bounds.width, 14)
                        .assign_to(&mut rect_opt1),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(display_bounds.width, 14)
                        .assign_to(&mut rect_opt2),
                );

            root.layout(display_bounds);

            ui.label(rect_title, "PONG GAME").center().draw();
            ui.label(rect_opt1, "[UP]  : 1P vs AI").center().draw();
            ui.label(rect_opt2, "[DOWN]: 1P vs 2P").center().draw();
        }
        GamePhase::Playing => {
            let mut rect_header = Rect::default();
            let mut rect_court = Rect::default();

            let root = FlexNode::new(FlexDirection::Column)
                .align_items(AlignItems::Stretch)
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(display_bounds.width, 12)
                        .assign_to(&mut rect_header),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_flex(1)
                        .assign_to(&mut rect_court),
                );

            root.layout(display_bounds);

            // Score Board Rendering Layout
            let mut score_bytes = [0u8; 16];
            let score_str = score_format(&mut score_bytes, state.player_score, state.enemy_score);
            ui.label(rect_header, score_str).center().draw();
            ui.draw_line(
                (rect_court.x, rect_court.y),
                (rect_court.x + rect_court.width as i32, rect_court.y),
                BinaryColor::On,
            );

            // Draw Player Left Paddle (using round for fluid visual positioning)
            ui.draw_filled_rect(
                Rect::new(
                    rect_court.x + 2,
                    rect_court.y + state.player_y.round() as i32,
                    2,
                    10,
                ),
                BinaryColor::On,
            );

            // Draw Enemy / P2 Right Paddle (using round for fluid visual positioning)
            ui.draw_filled_rect(
                Rect::new(
                    rect_court.x + rect_court.width as i32 - 4,
                    rect_court.y + state.enemy_y.round() as i32,
                    2,
                    10,
                ),
                BinaryColor::On,
            );

            // Draw Center Court Net Splitter
            for y in (rect_court.y..rect_court.y + rect_court.height as i32).step_by(6) {
                ui.draw_filled_rect(Rect::new(rect_court.x + 64, y, 1, 3), BinaryColor::On);
            }

            // Draw Ball Element Grid Matrix (using round to fix truncation judder)
            ui.draw_filled_rect(
                Rect::new(
                    rect_court.x + state.ball_x.round() as i32,
                    rect_court.y + state.ball_y.round() as i32,
                    2,
                    2,
                ),
                BinaryColor::On,
            );
        }
    }
}

fn score_format(buf: &mut [u8], p_score: u32, e_score: u32) -> &str {
    let mut idx = 0;

    // Parse Player Score metrics
    if p_score == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        let mut temp = p_score;
        let start = idx;
        while temp > 0 {
            buf[idx] = b'0' + (temp % 10) as u8;
            temp /= 10;
            idx += 1;
        }
        buf[start..idx].reverse();
    }

    // Insert structural divider space mapping boundaries
    buf[idx] = b' ';
    idx += 1;
    buf[idx] = b':';
    idx += 1;
    buf[idx] = b' ';
    idx += 1;

    // Parse Enemy Score metrics
    let enemy_start = idx;
    if e_score == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        let mut temp = e_score;
        while temp > 0 {
            buf[idx] = b'0' + (temp % 10) as u8;
            temp /= 10;
            idx += 1;
        }
        buf[enemy_start..idx].reverse();
    }

    core::str::from_utf8(&buf[..idx]).unwrap_or("0 : 0")
}
