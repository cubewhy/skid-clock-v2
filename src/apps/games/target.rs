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

const SCREEN_WIDTH: i32 = 128;
const SCREEN_HEIGHT: i32 = 64;
const PLAY_ZONE_Y: i32 = 10;
const TGT_DURATION_MS: u32 = 45000; // 45-second time limit per match

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GameTarget {
    pub x: f32,
    pub y: f32,
    pub max_radius: f32,
    pub current_radius: f32,
    pub spawn_time: u32,
    pub life_time: u32,
    pub active: bool,
}

pub struct TargetState {
    phase: GamePhase,
    targets: [GameTarget; 3],
    crosshair_x: f32,
    crosshair_y: f32,
    tgt_score: i32,
    tgt_shots_fired: i32,
    tgt_hits: i32,
    tgt_start_time: u32,
    last_update: Instant,
    base_time: Instant,
    last_hit_x: i32,
    last_hit_y: i32,
    hit_pulse_radius: i32,
    rng_seed: u32,
}

impl Default for TargetState {
    fn default() -> Self {
        let now = Instant::now();
        let mut state = Self {
            phase: GamePhase::Playing,
            targets: [GameTarget::default(); 3],
            crosshair_x: 64.0,
            crosshair_y: 37.0,
            tgt_score: 0,
            tgt_shots_fired: 0,
            tgt_hits: 0,
            tgt_start_time: 0,
            last_update: now,
            base_time: now,
            last_hit_x: -1,
            last_hit_y: -1,
            hit_pulse_radius: -1,
            rng_seed: 12345,
        };
        state.init_target_game();
        state
    }
}

impl TargetState {
    // Simple pseudo-random number generator mapping to Arduino's random()
    fn pseudo_rand(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let rand_val = ((self.rng_seed / 65536) % 32768) as i32;
        min + (rand_val % (max - min))
    }

    fn spawn_single_target(&mut self, idx: usize, now_ms: u32) {
        let max_r = self.pseudo_rand(6, 13) as f32;
        self.targets[idx].active = true;
        self.targets[idx].max_radius = max_r;
        self.targets[idx].current_radius = max_r;

        // Establish margin buffer restrictions to protect structural borders
        let min_x = (max_r + 2.0) as i32;
        let max_x = (SCREEN_WIDTH as f32 - max_r - 2.0) as i32;
        self.targets[idx].x = self.pseudo_rand(min_x, max_x) as f32;

        let min_y = (PLAY_ZONE_Y as f32 + max_r + 2.0) as i32;
        let max_y = (SCREEN_HEIGHT as f32 - max_r - 2.0) as i32;
        self.targets[idx].y = self.pseudo_rand(min_y, max_y) as f32;

        self.targets[idx].spawn_time = now_ms;
        self.targets[idx].life_time = self.pseudo_rand(2000, 4500) as u32;
    }

    fn init_target_game(&mut self) {
        let now = Instant::now();
        self.tgt_score = 0;
        self.tgt_shots_fired = 0;
        self.tgt_hits = 0;
        self.phase = GamePhase::Playing;
        self.crosshair_x = 64.0;
        self.crosshair_y = 37.0;

        self.last_hit_x = -1;
        self.last_hit_y = -1;
        self.hit_pulse_radius = -1;
        self.last_update = now;
        self.base_time = now;
        self.tgt_start_time = 0;

        for i in 0..3 {
            self.spawn_single_target(i, 0);
            // Stagger initial targets lifecycle parameters natively
            let offset = self.pseudo_rand(0, 1000) as u32;
            if self.targets[i].spawn_time >= offset {
                self.targets[i].spawn_time -= offset;
            }
        }
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut TargetState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::main_menu());
    }

    let now = Instant::now();
    let now_ms = now.duration_since(state.base_time).as_millis() as u32;

    if state.phase == GamePhase::GameOver {
        if ctx.input_manager.is_down(UiEvents::CONFIRM)
            || ctx.input_manager.is_down(UiEvents::KEY_7)
        {
            state.init_target_game();
        }
        return None;
    }

    let mut dt = now.duration_since(state.last_update).as_secs_f32() * 1000.0 / 25.0;
    state.last_update = now;
    if dt > 3.0 {
        dt = 3.0;
    }

    // 1. Match Countdown Resolution Check
    let elapsed_match = now_ms - state.tgt_start_time;
    let time_left = (TGT_DURATION_MS as i32 - elapsed_match as i32) / 1000;
    if time_left <= 0 {
        state.phase = GamePhase::GameOver;
        return None;
    }

    // 2. Crosshair Multidirectional Machine Maneuvering Logic via Joystick Inputs
    let move_speed = 1.8 * dt;
    if ctx.input_manager.is_down(UiEvents::LEFT) {
        state.crosshair_x -= move_speed;
    } else if ctx.input_manager.is_down(UiEvents::RIGHT) {
        state.crosshair_x += move_speed;
    }
    if ctx.input_manager.is_down(UiEvents::UP) {
        state.crosshair_y -= move_speed;
    } else if ctx.input_manager.is_down(UiEvents::DOWN) {
        state.crosshair_y += move_speed;
    }

    // Secure screen interaction boundary safe wall bounds
    state.crosshair_x = state.crosshair_x.clamp(2.0, (SCREEN_WIDTH - 3) as f32);
    state.crosshair_y = state
        .crosshair_y
        .clamp((PLAY_ZONE_Y + 2) as f32, (SCREEN_HEIGHT - 3) as f32);

    // 3. Targets Lifespan Evaluation & Degradation Processing
    for i in 0..3 {
        if !state.targets[i].active {
            state.spawn_single_target(i, now_ms);
            continue;
        }

        let elapsed_target = now_ms.saturating_sub(state.targets[i].spawn_time);
        if elapsed_target >= state.targets[i].life_time {
            // Target timeout penalty execution step (-10 points)
            state.tgt_score = (state.tgt_score - 10).clamp(0, 9999);
            state.spawn_single_target(i, now_ms);
        } else {
            // Linearly shrink target tracking profile volume over runtime scaling metrics
            let life_ratio = elapsed_target as f32 / state.targets[i].life_time as f32;
            state.targets[i].current_radius =
                state.targets[i].max_radius * (1.0 - life_ratio * 0.4);
        }
    }

    // 4. Trigger Pull Shooting Detection (Triggered via CONFIRM or KEY_7)
    let is_shooting =
        ctx.input_manager.is_down(UiEvents::CONFIRM) || ctx.input_manager.is_down(UiEvents::KEY_7);
    if is_shooting && !ctx.menu_events.contains(UiEvents::KEY_ESC) {
        // Simple click latch tracking emulation if required (assuming standard input tick gating)
        state.tgt_shots_fired += 1;
        let mut any_hit = false;
        let mut closest_dist = 999.0;
        let mut hit_idx: i32 = -1;

        for i in 0..3 {
            if !state.targets[i].active {
                continue;
            }
            let dist = ((state.crosshair_x - state.targets[i].x).powi(2)
                + (state.crosshair_y - state.targets[i].y).powi(2))
            .sqrt();

            if dist <= state.targets[i].current_radius && dist < closest_dist {
                closest_dist = dist;
                hit_idx = i as i32;
                any_hit = true;
            }
        }

        if any_hit && hit_idx != -1 {
            state.tgt_hits += 1;
            let idx = hit_idx as usize;

            // Proportional precision mapping calculation layer
            let perfection = 1.0 - (closest_dist / state.targets[idx].current_radius);
            let points_earned = 50 + (perfection * 50.0) as i32;
            state.tgt_score += points_earned;

            // Initialize shockwave particle ripple feedback data
            state.last_hit_x = state.targets[idx].x as i32;
            state.last_hit_y = state.targets[idx].y as i32;
            state.hit_pulse_radius = 0;

            state.targets[idx].active = false;
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &mut TargetState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let now_ms = Instant::now().duration_since(state.base_time).as_millis() as u32;

    if state.phase == GamePhase::GameOver {
        let mut rect_title = Rect::default();
        let mut rect_score = Rect::default();
        let mut rect_accuracy = Rect::default();
        let mut rect_hint = Rect::default();

        let root = FlexNode::new(FlexDirection::Column)
            .align_items(AlignItems::Center)
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 16)
                    .assign_to(&mut rect_title),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 12)
                    .assign_to(&mut rect_score),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 12)
                    .assign_to(&mut rect_accuracy),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 12)
                    .assign_to(&mut rect_hint),
            );

        root.layout(display_bounds);

        ui.label(rect_title, "TIME'S UP").center().draw();

        let mut score_bytes = [0u8; 16];
        let score_str = results_format(&mut score_bytes, "Score: ", state.tgt_score);
        ui.label(rect_score, score_str).center().draw();

        let accuracy = if state.tgt_shots_fired > 0 {
            (state.tgt_hits * 100) / state.tgt_shots_fired
        } else {
            0
        };
        let mut acc_bytes = [0u8; 20];
        let acc_str = results_format(&mut acc_bytes, "Accuracy: ", accuracy);
        // Note: Manual append '%' label behavior or clean formatting sequence
        ui.label(rect_accuracy, acc_str).center().draw();

        ui.label(rect_hint, "[PRESS CONFIRM] MENU").center().draw();
        return;
    }

    // Draw Dashboard Context Header Panel Data
    let elapsed = now_ms - state.tgt_start_time;
    let time_left = ((TGT_DURATION_MS as i32 - elapsed as i32) / 1000).max(0);

    let mut hud_bytes = [0u8; 32];
    let hud_str = hud_format(&mut hud_bytes, state.tgt_score, time_left);
    ui.label(Rect::new(2, 0, 124, 10), hud_str).draw();
    ui.draw_line((0, 9), (SCREEN_WIDTH, 9), BinaryColor::On);

    // A. Draw concentric rings targeting nodes
    for i in 0..3 {
        if !state.targets[i].active {
            continue;
        }
        let tx = state.targets[i].x as i32;
        let ty = state.targets[i].y as i32;
        let r = state.targets[i].current_radius as i32;

        ui.draw_procedural_circle(tx, ty, r);
        if r > 5 {
            ui.draw_procedural_circle(tx, ty, r / 2);
        }
        ui.draw_filled_rect(Rect::new(tx, ty, 1, 1), BinaryColor::On);
    }

    // B. Handle asynchronous animated shockwave expand rings
    let mut current_pulse = state.hit_pulse_radius;
    if current_pulse >= 0 {
        ui.draw_procedural_circle(state.last_hit_x, state.last_hit_y, current_pulse);
        ui.label(
            Rect::new(state.last_hit_x - 8, state.last_hit_y - 12, 24, 10),
            "HIT!",
        )
        .draw();

        // Step speed regulation handling natively via ticks matrix offsets
        if now_ms.is_multiple_of(2) {
            current_pulse += 2;
        }
        // Force state alteration safely by replicating the mutable pointer change
        if current_pulse > 10 {
            state.hit_pulse_radius = -1;
        } else {
            state.hit_pulse_radius = current_pulse;
        }
    }

    // C. Draw Crosshair Target HUD Component
    let cx = state.crosshair_x as i32;
    let cy = state.crosshair_y as i32;
    ui.draw_procedural_circle(cx, cy, 3);
    ui.draw_line((cx - 6, cy), (cx + 6, cy), BinaryColor::On);
    ui.draw_line((cx, cy - 6), (cx, cy + 6), BinaryColor::On);
}

fn hud_format(buf: &mut [u8], score: i32, time_left: i32) -> &str {
    let mut idx = 0;
    let lbl_score = b"SCORE:";
    buf[idx..idx + lbl_score.len()].copy_from_slice(lbl_score);
    idx += lbl_score.len();

    let score_start = idx;
    let mut temp_s = score.max(0);
    if temp_s == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        while temp_s > 0 {
            buf[idx] = b'0' + (temp_s % 10) as u8;
            temp_s /= 10;
            idx += 1;
        }
        buf[score_start..idx].reverse();
    }

    while idx < 16 {
        buf[idx] = b' ';
        idx += 1;
    }

    let lbl_time = b"TIME:";
    buf[idx..idx + lbl_time.len()].copy_from_slice(lbl_time);
    idx += lbl_time.len();

    let time_start = idx;
    let mut temp_t = time_left;
    if temp_t == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        while temp_t > 0 {
            buf[idx] = b'0' + (temp_t % 10) as u8;
            temp_t /= 10;
            idx += 1;
        }
        buf[time_start..idx].reverse();
    }

    buf[idx] = b's';
    idx += 1;

    core::str::from_utf8(&buf[..idx]).unwrap_or("SCORE:0 TIME:0s")
}

fn results_format<'a>(buf: &'a mut [u8], prefix: &'a str, value: i32) -> &'a str {
    let mut idx = 0;
    let p_bytes = prefix.as_bytes();
    buf[idx..idx + p_bytes.len()].copy_from_slice(p_bytes);
    idx += p_bytes.len();

    let start = idx;
    let mut temp = value.max(0);
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

    if prefix.starts_with("Accuracy") {
        buf[idx] = b'%';
        idx += 1;
    }

    core::str::from_utf8(&buf[..idx]).unwrap_or("")
}
