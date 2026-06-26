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

const MAX_CHARGE: f32 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JumpState {
    Idle,
    Charge,
    Flight,
    Scroll,
    GameOver,
}

pub struct JumpJumpState {
    phase: JumpState,
    player_x: f32,
    player_y: f32,
    jump_start_x: f32,
    jump_target_x: f32,
    flight_progress: f32,
    cur_plat_x: f32,
    cur_plat_w: f32,
    next_plat_x: f32,
    next_plat_w: f32,
    charge_power: f32,
    jj_score: i32,
    last_update: Instant,
    base_time: Instant,
    input_locked: bool,
    rng_seed: u32,
}

impl Default for JumpJumpState {
    fn default() -> Self {
        let now = Instant::now();
        let mut state = Self {
            phase: JumpState::Idle,
            player_x: 20.0,
            player_y: 50.0,
            jump_start_x: 0.0,
            jump_target_x: 0.0,
            flight_progress: 0.0,
            cur_plat_x: 10.0,
            cur_plat_w: 20.0,
            next_plat_x: 0.0,
            next_plat_w: 0.0,
            charge_power: 0.0,
            jj_score: 0,
            last_update: now,
            base_time: now,
            input_locked: true,
            rng_seed: 54321,
        };
        state.init_game();
        state
    }
}

impl JumpJumpState {
    // Linear Congruential Generator matching Arduino's random() logic
    fn pseudo_rand(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let rand_val = ((self.rng_seed / 65536) % 32768) as i32;
        min + (rand_val % (max - min))
    }

    fn init_game(&mut self) {
        let now = Instant::now();
        self.jj_score = 0;
        self.phase = JumpState::Idle;
        self.player_x = 20.0;
        self.player_y = 50.0;
        self.cur_plat_x = 10.0;
        self.cur_plat_w = 20.0;
        self.next_plat_x = self.pseudo_rand(55, 90) as f32;
        self.next_plat_w = self.pseudo_rand(12, 22) as f32;
        self.charge_power = 0.0;
        self.last_update = now;
        self.base_time = now;
        self.input_locked = true;
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut JumpJumpState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    if state.phase == JumpState::GameOver {
        if ctx.input_manager.is_down(UiEvents::CONFIRM) || ctx.input_manager.is_down(UiEvents::UP) {
            state.init_game();
        }
        return None;
    }

    let now = Instant::now();
    let mut dt = now.duration_since(state.last_update).as_secs_f32() * 1000.0 / 25.0;
    state.last_update = now;
    if dt > 3.0 {
        dt = 3.0;
    }

    // Evaluate input redirection (Joystick push detection across directions)
    let joy_active = ctx.input_manager.is_down(UiEvents::UP)
        || ctx.input_manager.is_down(UiEvents::DOWN)
        || ctx.input_manager.is_down(UiEvents::LEFT)
        || ctx.input_manager.is_down(UiEvents::RIGHT);

    // Filter latched inputs to guard system actions until inputs clear out
    let mut processed_joy_active = joy_active;
    if state.input_locked {
        if !joy_active {
            state.input_locked = false;
        }
        processed_joy_active = false;
    }

    match state.phase {
        JumpState::Idle => {
            if processed_joy_active {
                state.phase = JumpState::Charge;
                state.charge_power = 0.0;
            }
        }
        JumpState::Charge => {
            if processed_joy_active {
                state.charge_power += 1.8 * dt;
                if state.charge_power > MAX_CHARGE {
                    state.charge_power = MAX_CHARGE;
                }
            } else {
                // Key/Joystick release triggers physics mechanics jump flight path
                state.phase = JumpState::Flight;
                state.jump_start_x = state.player_x;
                state.jump_target_x = state.player_x + state.charge_power * 1.3;
                state.flight_progress = 0.0;
            }
        }
        JumpState::Flight => {
            state.flight_progress += 0.06 * dt;
            if state.flight_progress >= 1.0 {
                state.flight_progress = 1.0;
                state.player_x = state.jump_target_x;
                state.player_y = 50.0;

                // Evaluate landing destination bounds alignment checks
                if state.player_x >= state.next_plat_x
                    && state.player_x <= (state.next_plat_x + state.next_plat_w)
                {
                    state.jj_score += 1;
                    state.phase = JumpState::Scroll;
                } else if state.player_x >= state.cur_plat_x
                    && state.player_x <= (state.cur_plat_x + state.cur_plat_w)
                {
                    // Jumped right back into initial departure platform safely
                    state.phase = JumpState::Idle;
                } else {
                    state.phase = JumpState::GameOver;
                }
            } else {
                // Parabolic projectile simulation trajectory execution path
                state.player_x = state.jump_start_x
                    + (state.jump_target_x - state.jump_start_x) * state.flight_progress;
                state.player_y =
                    50.0 - (state.flight_progress * core::f32::consts::PI).sin() * 22.0;
            }
        }
        JumpState::GameOver => {}
        JumpState::Scroll => {
            // Smoothly move perspective horizontally to transition screen viewport frame elements
            let scroll_speed = 3.0 * dt;
            state.player_x -= scroll_speed;
            state.cur_plat_x -= scroll_speed;
            state.next_plat_x -= scroll_speed;

            if state.next_plat_x <= 20.0 {
                let diff = 20.0 - state.next_plat_x;
                state.player_x += diff;
                state.next_plat_x = 20.0;
                state.cur_plat_x = state.next_plat_x;
                state.cur_plat_w = state.next_plat_w;

                // Procedurally spawn next landing objective in deep background buffer right boundaries
                state.next_plat_x = state.pseudo_rand(60, 100) as f32;
                state.next_plat_w = state.pseudo_rand(12, 22) as f32;
                state.phase = JumpState::Idle;
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &JumpJumpState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    if state.phase == JumpState::GameOver {
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

        ui.label(rect_title, "FALL DOWN").center().draw();

        let mut score_bytes = [0u8; 16];
        let score_str = score_format(&mut score_bytes, "Score: ", state.jj_score);
        ui.label(rect_score, score_str).center().draw();
        ui.label(rect_hint, "[PRESS UP TO RETRY]").center().draw();
        return;
    }

    // Display Header Context Data Panel
    let mut hud_bytes = [0u8; 16];
    let hud_str = score_format(&mut hud_bytes, "Score: ", state.jj_score);
    ui.label(Rect::new(2, 0, 124, 10), hud_str).draw();
    ui.draw_line((0, 9), (128, 9), BinaryColor::On);

    // Draw solid landing target platforms
    ui.draw_filled_rect(
        Rect::new(state.cur_plat_x as i32, 52, state.cur_plat_w as u32, 12),
        BinaryColor::On,
    );
    ui.draw_filled_rect(
        Rect::new(state.next_plat_x as i32, 52, state.next_plat_w as u32, 12),
        BinaryColor::On,
    );

    // Render interactive charging metrics indicator modules
    if state.phase == JumpState::Charge {
        let bar_width = ((state.charge_power / MAX_CHARGE) * 40.0) as i32;
        let bar_start_x = 60;

        ui.draw_stroke_rect(Rect::new(bar_start_x, 2, 42, 5), BinaryColor::On, 1);
        ui.draw_filled_rect(
            Rect::new(bar_start_x + 1, 3, bar_width as u32, 3),
            BinaryColor::On,
        );

        #[cfg(feature = "jump_jump_cheat")]
        {
            // Projectile trajectory projection dots
            let proj_target_x = state.player_x + state.charge_power * 1.3;
            let mut t = 0.15;
            while t < 1.0 {
                let proj_x = state.player_x + (proj_target_x - state.player_x) * t;
                let proj_y = 50.0 - (t * core::f32::consts::PI).sin() * 22.0;
                ui.draw_filled_rect(
                    Rect::new(proj_x as i32, proj_y as i32, 1, 1),
                    BinaryColor::On,
                );
                t += 0.15;
            }
        }
    }

    // Render Player core bounding entity box (3x3 Solid Block Asset)
    ui.draw_filled_rect(
        Rect::new(state.player_x as i32 - 1, state.player_y as i32 - 3, 3, 3),
        BinaryColor::On,
    );
}

fn score_format<'a>(buf: &'a mut [u8], prefix: &'a str, score: i32) -> &'a str {
    let mut idx = 0;
    let p_bytes = prefix.as_bytes();
    buf[idx..idx + p_bytes.len()].copy_from_slice(p_bytes);
    idx += p_bytes.len();

    let start = idx;
    let mut temp = score.max(0);
    if temp == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        while temp > 0 {
            buf[idx] = b'0' + (temp % 10) as u8;
            temp /= 10;
            idx += 1;
        }
        buf[start..idx].reverse();
    }

    core::str::from_utf8(&buf[..idx]).unwrap_or("")
}
