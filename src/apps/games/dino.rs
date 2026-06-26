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
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Obstacle {
    pub active: bool,
    pub x: f32,
    pub y: i32,
    pub obstacle_type: i32,
}

pub struct DinoState {
    phase: GamePhase,
    dino_y: f32,
    dino_vy: f32,
    is_jumping: bool,
    is_ducking: bool,
    score: f32,
    game_speed: f32,
    last_update: Instant,
    base_time: Instant,
    last_obstacle_spawn: u32,
    obstacles: [Obstacle; 2],
}

impl Default for DinoState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            phase: GamePhase::Playing,
            dino_y: 40.0,
            dino_vy: 0.0,
            is_jumping: false,
            is_ducking: false,
            score: 0.0,
            game_speed: 0.10,
            last_update: now,
            base_time: now,
            last_obstacle_spawn: 0,
            obstacles: [Obstacle::default(); 2],
        }
    }
}

impl DinoState {
    fn reset_game(&mut self) {
        let now = Instant::now();
        self.dino_y = 40.0;
        self.dino_vy = 0.0;
        self.is_jumping = false;
        self.is_ducking = false;
        self.score = 0.0;
        self.game_speed = 0.10;
        self.last_update = now;
        self.base_time = now;
        self.last_obstacle_spawn = 0;
        for i in 0..2 {
            self.obstacles[i].active = false;
        }
        self.phase = GamePhase::Playing;
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut DinoState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    let now = Instant::now();
    let now_ms = now.duration_since(state.base_time).as_millis() as u32;

    if state.phase == GamePhase::GameOver {
        if ctx.input_manager.is_down(UiEvents::KEY_7) || ctx.input_manager.is_down(UiEvents::UP) {
            state.reset_game();
        }
        return None;
    }

    let mut dt = now.duration_since(state.last_update).as_secs_f32() * 1000.0;
    state.last_update = now;

    dt = dt.clamp(0.0, 50.0);
    if dt == 0.0 {
        return None;
    }

    // Input state decoding logic matching joystick axes
    if ctx.input_manager.is_down(UiEvents::DOWN | UiEvents::KEY_5) {
        if !state.is_jumping {
            state.is_ducking = true;
        } else {
            state.dino_vy += 0.00055 * 2.5 * dt;
        }
    } else {
        state.is_ducking = false;
    }

    if ctx.input_manager.is_down(UiEvents::UP | UiEvents::KEY_6)
        && !state.is_jumping
        && !state.is_ducking
    {
        state.dino_vy = -0.17;
        state.is_jumping = true;
    }

    // Core progression acceleration step updates
    state.score += 0.015 * dt;
    state.game_speed += 0.000002 * dt;
    if state.game_speed > 0.22 {
        state.game_speed = 0.22;
    }

    if state.is_jumping {
        state.dino_y += state.dino_vy * dt;
        state.dino_vy += 0.00055 * dt;
        if state.dino_y >= 40.0 {
            state.dino_y = 40.0;
            state.is_jumping = false;
            state.dino_vy = 0.0;
        }
    }

    let mut obstacle_active = false;
    for i in 0..2 {
        if state.obstacles[i].active {
            state.obstacles[i].x -= state.game_speed * dt;
            if state.obstacles[i].x < -12.0 {
                state.obstacles[i].active = false;
            } else {
                obstacle_active = true;

                // Hitbox configuration matrix values
                let dw = if state.is_ducking { 14 } else { 10 };
                let dh = if state.is_ducking { 6 } else { 12 };
                let dy = if state.is_ducking {
                    46
                } else {
                    state.dino_y.round() as i32
                };
                let dx = 15;

                let ow = match state.obstacles[i].obstacle_type {
                    0 => 6,
                    1 => 8,
                    _ => 12,
                };
                let oh = match state.obstacles[i].obstacle_type {
                    0 => 10,
                    1 => 14,
                    _ => 6,
                };

                let ox = state.obstacles[i].x.round() as i32;
                let oy = state.obstacles[i].y;

                // Axis-Aligned Bounding Box overlapping check
                if dx < ox + ow && dx + dw > ox && dy < oy + oh && dy + dh > oy {
                    state.phase = GamePhase::GameOver;
                    return None;
                }
            }
        }
    }

    let mut spawn_interval = (1500.0 / (state.game_speed / 0.10)) as u32;
    if spawn_interval < 600 {
        spawn_interval = 600;
    }

    if !obstacle_active && (now_ms - state.last_obstacle_spawn > spawn_interval) {
        for i in 0..2 {
            if !state.obstacles[i].active {
                state.obstacles[i].active = true;
                state.obstacles[i].x = 128.0;

                // Generate a reliable pseudo-random type identifier using monotonic tickers
                let pseudo_rand = ((now_ms ^ state.score as u32) % 4) as i32;
                state.obstacles[i].obstacle_type = pseudo_rand;

                state.obstacles[i].y = match pseudo_rand {
                    0 => 42,
                    1 => 38,
                    2 => 44, // Low Altitude Pterodactyl path
                    _ => 39, // High Altitude Pterodactyl path
                };

                state.last_obstacle_spawn = now_ms;
                break;
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &DinoState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let now_ms = Instant::now().duration_since(state.base_time).as_millis() as u32;

    if state.phase == GamePhase::GameOver {
        let mut rect_title = Rect::default();
        let mut rect_score = Rect::default();
        let mut rect_hint = Rect::default();

        let root = FlexNode::new(FlexDirection::Column)
            .align_items(AlignItems::Center)
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 18)
                    .assign_to(&mut rect_title),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 14)
                    .assign_to(&mut rect_score),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 14)
                    .assign_to(&mut rect_hint),
            );

        root.layout(display_bounds);

        ui.label(rect_title, "GAME OVER").center().draw();

        let mut score_bytes = [0u8; 16];
        let score_str = hud_format(&mut score_bytes, state.score as u32);
        ui.label(rect_score, score_str).center().draw();
        ui.label(rect_hint, "[PRESS UP TO RETRY]").center().draw();
        return;
    }

    // Horizon line drawing frame element
    ui.draw_line((0, 52), (128, 52), BinaryColor::On);

    // Procedural line/rect vector rendering for Player Dino character
    let px = 15;
    if state.is_ducking {
        let py = 46;
        ui.draw_filled_rect(Rect::new(px, py + 2, 9, 4), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(px + 6, py, 8, 4), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(px + 10, py + 1, 1, 1), BinaryColor::Off);

        if (now_ms / 70).is_multiple_of(2) {
            ui.draw_filled_rect(Rect::new(px + 3, py + 5, 1, 1), BinaryColor::Off);
            ui.draw_filled_rect(Rect::new(px + 6, py + 5, 1, 1), BinaryColor::On);
        } else {
            ui.draw_filled_rect(Rect::new(px + 3, py + 5, 1, 1), BinaryColor::On);
            ui.draw_filled_rect(Rect::new(px + 6, py + 5, 1, 1), BinaryColor::Off);
        }
    } else {
        let py = state.dino_y.round() as i32;
        ui.draw_filled_rect(Rect::new(px + 4, py, 6, 4), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(px + 6, py + 1, 1, 1), BinaryColor::Off);
        ui.draw_filled_rect(Rect::new(px + 4, py + 4, 3, 3), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(px + 1, py + 6, 8, 4), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(px, py + 6, 1, 1), BinaryColor::On);

        if state.is_jumping {
            ui.draw_line((px + 3, py + 10), (px + 3, py + 11), BinaryColor::On);
            ui.draw_line((px + 6, py + 10), (px + 6, py + 11), BinaryColor::On);
        } else {
            if (now_ms / 90).is_multiple_of(2) {
                ui.draw_line((px + 3, py + 10), (px + 3, py + 11), BinaryColor::On);
                ui.draw_filled_rect(Rect::new(px + 6, py + 10, 1, 1), BinaryColor::On);
            } else {
                ui.draw_filled_rect(Rect::new(px + 3, py + 10, 1, 1), BinaryColor::On);
                ui.draw_line((px + 6, py + 10), (px + 6, py + 11), BinaryColor::On);
            }
        }
    }

    // Procedural rendering cycle for Obstacles and Flying Pterodactyls
    for i in 0..2 {
        if state.obstacles[i].active {
            let ox = state.obstacles[i].x.round() as i32;
            let oy = state.obstacles[i].y;

            if state.obstacles[i].obstacle_type == 0 {
                ui.draw_filled_rect(Rect::new(ox + 2, oy, 2, 10), BinaryColor::On);
                ui.draw_line((ox, oy + 3), (ox, oy + 6), BinaryColor::On);
                ui.draw_filled_rect(Rect::new(ox + 1, oy + 3, 1, 1), BinaryColor::On);
                ui.draw_line((ox + 5, oy + 1), (ox + 5, oy + 4), BinaryColor::On);
                ui.draw_filled_rect(Rect::new(ox + 4, oy + 1, 1, 1), BinaryColor::On);
            } else if state.obstacles[i].obstacle_type == 1 {
                ui.draw_filled_rect(Rect::new(ox + 3, oy, 2, 14), BinaryColor::On);
                ui.draw_line((ox + 1, oy + 4), (ox + 1, oy + 8), BinaryColor::On);
                ui.draw_line((ox + 1, oy + 4), (ox + 2, oy + 4), BinaryColor::On);
                ui.draw_line((ox + 6, oy + 2), (ox + 6, oy + 7), BinaryColor::On);
                ui.draw_line((ox + 5, oy + 2), (ox + 6, oy + 2), BinaryColor::On);
            } else {
                // Flying Pterodactyl wing transformation animation branch
                ui.draw_filled_rect(Rect::new(ox + 3, oy + 2, 5, 2), BinaryColor::On);
                ui.draw_filled_rect(Rect::new(ox + 8, oy + 1, 1, 1), BinaryColor::On);
                ui.draw_line((ox + 8, oy + 2), (ox + 11, oy + 2), BinaryColor::On);
                ui.draw_filled_rect(Rect::new(ox + 2, oy + 3, 1, 1), BinaryColor::On);

                if (now_ms / 130).is_multiple_of(2) {
                    ui.draw_line((ox + 4, oy + 2), (ox + 2, oy), BinaryColor::On);
                    ui.draw_line((ox + 5, oy + 2), (ox + 7, oy), BinaryColor::On);
                } else {
                    ui.draw_line((ox + 4, oy + 3), (ox + 2, oy + 5), BinaryColor::On);
                    ui.draw_line((ox + 5, oy + 3), (ox + 7, oy + 5), BinaryColor::On);
                }
            }
        }
    }

    // Metrics overlay tracking HUD element layout
    let rect_hud = Rect::new(95, 2, 30, 10);
    let mut score_bytes = [0u8; 12];
    let score_str = score_format(&mut score_bytes, state.score as u32);
    ui.label(rect_hud, score_str).draw();
}

fn hud_format(buf: &mut [u8], score: u32) -> &str {
    let mut idx = 0;
    let label = b"Score: ";
    buf[idx..idx + label.len()].copy_from_slice(label);
    idx += label.len();

    let score_start = idx;
    if score == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        let mut temp = score;
        while temp > 0 {
            buf[idx] = b'0' + (temp % 10) as u8;
            temp /= 10;
            idx += 1;
        }
        buf[score_start..idx].reverse();
    }
    core::str::from_utf8(&buf[..idx]).unwrap_or("Score: 0")
}

fn score_format(buf: &mut [u8], score: u32) -> &str {
    let mut idx = 0;
    if score == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        let mut temp = score;
        while temp > 0 {
            buf[idx] = b'0' + (temp % 10) as u8;
            temp /= 10;
            idx += 1;
        }
        buf[0..idx].reverse();
    }
    core::str::from_utf8(&buf[..idx]).unwrap_or("0")
}
