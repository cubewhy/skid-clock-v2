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

const TOTAL_MINERALS: usize = 8;
const LEVEL_DURATION_SECS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MinerState {
    Swing,
    Extend,
    Retract,
}

#[derive(Debug, Clone, Copy)]
struct GoldItem {
    x: f32,
    y: f32,
    radius: f32,
    value: u32,
    weight: f32,
    active: bool,
}

impl Default for GoldItem {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            radius: 0.0,
            value: 0,
            weight: 1.0,
            active: false,
        }
    }
}

pub struct GoldMinerState {
    miner_state: MinerState,
    hook_angle: f32,
    hook_speed: f32,
    hook_length: f32,
    minerals: [GoldItem; TOTAL_MINERALS],
    score: u32,
    grabbed_idx: Option<usize>,
    level: u32,
    target_score: u32,
    level_start_time: Instant,
    last_tick: Instant,
    is_game_over: bool,
    is_level_complete: bool,
    rng: SmallRng,
}

impl Default for GoldMinerState {
    fn default() -> Self {
        let rng: SmallRng = rand::make_rng();

        let mut state = Self {
            miner_state: MinerState::Swing,
            hook_angle: 0.0,
            hook_speed: 0.04,
            hook_length: 10.0,
            minerals: [GoldItem::default(); TOTAL_MINERALS],
            score: 0,
            grabbed_idx: None,
            level: 1,
            target_score: 0,
            level_start_time: Instant::now(),
            last_tick: Instant::now(),
            is_game_over: false,
            is_level_complete: false,
            rng,
        };

        let current_map_value = state.generate_minerals();
        state.target_score = (current_map_value as f32 * 0.50) as u32;
        state
    }
}

impl GoldMinerState {
    fn generate_minerals(&mut self) -> u32 {
        let mut total_val = 0;
        for i in 0..TOTAL_MINERALS {
            self.minerals[i].active = true;

            let mut safety_counter = 0;
            loop {
                self.minerals[i].x = self.rng.random_range(10..118) as f32;
                self.minerals[i].y = self.rng.random_range(28..56) as f32;

                let type_chance = self.rng.random_range(0..10);
                if type_chance < 5 {
                    // Stone configuration
                    self.minerals[i].radius = self.rng.random_range(4..7) as f32;
                    self.minerals[i].value = self.rng.random_range(15..40);
                    self.minerals[i].weight = self.rng.random_range(4..6) as f32;
                } else if type_chance < 9 {
                    // Gold configuration
                    self.minerals[i].radius = self.rng.random_range(3..6) as f32;
                    self.minerals[i].value = (self.minerals[i].radius * 70.0) as u32;
                    self.minerals[i].weight = self.rng.random_range(2..4) as f32;
                } else {
                    // Diamond configuration
                    self.minerals[i].radius = 2.0;
                    self.minerals[i].value = 400;
                    self.minerals[i].weight = 1.0;
                }

                // Prevent dynamic spawn overlapping constraints
                let mut overlap = false;
                for j in 0..i {
                    let dist = ((self.minerals[i].x - self.minerals[j].x).powi(2)
                        + (self.minerals[i].y - self.minerals[j].y).powi(2))
                    .sqrt();
                    if dist < (self.minerals[i].radius + self.minerals[j].radius + 4.0) {
                        overlap = true;
                        break;
                    }
                }

                safety_counter += 1;
                if !overlap || safety_counter > 15 {
                    break;
                }
            }
            total_val += self.minerals[i].value;
        }
        total_val
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut GoldMinerState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    // === Game Over Screen Interceptor ===
    if state.is_game_over {
        if ctx.menu_events.contains(UiEvents::KEY_7) || ctx.menu_events.contains(UiEvents::CONFIRM)
        {
            *state = GoldMinerState::default();
        }
        return None;
    }

    // === Level Transition Complete Interceptor ===
    if state.is_level_complete {
        let action_triggered = ctx.menu_events.contains(UiEvents::KEY_7)
            || ctx.menu_events.contains(UiEvents::CONFIRM)
            || ctx.input_manager.is_down(UiEvents::LEFT)
            || ctx.input_manager.is_down(UiEvents::RIGHT)
            || ctx.input_manager.is_down(UiEvents::UP)
            || ctx.input_manager.is_down(UiEvents::DOWN);

        if action_triggered {
            state.level += 1;
            let current_map_value = state.generate_minerals();

            let mut target_ratio = 0.50 + (state.level as f32 * 0.02);
            if target_ratio > 0.75 {
                target_ratio = 0.75;
            }

            state.target_score += (current_map_value as f32 * target_ratio) as u32;

            // Anti-softlock safety protection rule
            if state.target_score <= state.score {
                state.target_score = state.score + (current_map_value as f32 * 0.30) as u32;
            }

            state.miner_state = MinerState::Swing;
            state.hook_angle = 0.0;
            state.hook_length = 10.0;
            state.grabbed_idx = None;
            state.is_level_complete = false;
            state.level_start_time = Instant::now();
        }
        return None;
    }

    // === Core Clock Countdown Metrics Updates ===
    let time_passed = state.level_start_time.elapsed().as_secs();
    let time_left = LEVEL_DURATION_SECS.saturating_sub(time_passed);

    if time_left == 0 {
        if state.score >= state.target_score {
            state.is_level_complete = true;
        } else {
            state.is_game_over = true;
        }
        return None;
    }

    // Process kinematic steps using frame scaling delta updates
    let elapsed = state.last_tick.elapsed().as_millis();
    if elapsed == 0 {
        return None;
    }
    state.last_tick = Instant::now();
    let mut dt = (elapsed as f32) / 25.0;
    if dt > 3.0 {
        dt = 3.0;
    }

    let input_active = ctx.input_manager.is_down(UiEvents::LEFT)
        || ctx.input_manager.is_down(UiEvents::RIGHT)
        || ctx.input_manager.is_down(UiEvents::UP)
        || ctx.input_manager.is_down(UiEvents::DOWN)
        || ctx.menu_events.contains(UiEvents::KEY_7)
        || ctx.menu_events.contains(UiEvents::CONFIRM);

    // Hook Physics Processing Finite State Machine
    match state.miner_state {
        MinerState::Swing => {
            state.hook_angle += state.hook_speed * dt;
            if state.hook_angle > 1.3 || state.hook_angle < -1.3 {
                state.hook_speed = -state.hook_speed;
            }
            if input_active {
                state.miner_state = MinerState::Extend;
            }
        }
        MinerState::Extend => {
            state.hook_length += 2.2 * dt; // launchSpeed
            let curr_x = 64.0 + state.hook_angle.sin() * state.hook_length;
            let curr_y = 14.0 + state.hook_angle.cos() * state.hook_length;

            if !(0.0..=128.0).contains(&curr_x) || curr_y > 64.0 {
                state.miner_state = MinerState::Retract;
                state.grabbed_idx = None;
            }

            for i in 0..TOTAL_MINERALS {
                if state.minerals[i].active {
                    let distance = ((curr_x - state.minerals[i].x).powi(2)
                        + (curr_y - state.minerals[i].y).powi(2))
                    .sqrt();
                    if distance < state.minerals[i].radius + 2.0 {
                        state.miner_state = MinerState::Retract;
                        state.grabbed_idx = Some(i);
                        break;
                    }
                }
            }
        }
        MinerState::Retract => {
            let mut curr_retract_speed = 2.0; // retractBaseSpeed
            if let Some(idx) = state.grabbed_idx {
                curr_retract_speed /= state.minerals[idx].weight;
                if curr_retract_speed < 0.3 {
                    curr_retract_speed = 0.3;
                }
            }

            state.hook_length -= curr_retract_speed * dt;

            if let Some(idx) = state.grabbed_idx {
                state.minerals[idx].x = 64.0 + state.hook_angle.sin() * state.hook_length;
                state.minerals[idx].y = 14.0 + state.hook_angle.cos() * state.hook_length;
            }

            if state.hook_length <= 10.0 {
                state.hook_length = 10.0;
                state.miner_state = MinerState::Swing;

                if let Some(idx) = state.grabbed_idx {
                    state.score += state.minerals[idx].value;
                    state.minerals[idx].active = false;
                    state.grabbed_idx = None;

                    // Evaluate map clearance check
                    let mut any_mineral_left = false;
                    for mineral in &state.minerals {
                        if mineral.active {
                            any_mineral_left = true;
                            break;
                        }
                    }
                    if !any_mineral_left {
                        if state.score >= state.target_score {
                            state.is_level_complete = true;
                        } else {
                            state.is_game_over = true;
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &GoldMinerState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut rect_lvl = Rect::default();
    let mut rect_tgt = Rect::default();
    let mut rect_time = Rect::default();
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
                        .assign_to(&mut rect_tgt),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_flex(1)
                        .assign_to(&mut rect_time),
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

    // Top Stats Line Processing
    let mut lvl_bytes = [0u8; 8];
    let lvl_str = msg_format(&mut lvl_bytes, "L:", state.level);
    ui.label(rect_lvl, lvl_str).center().draw();

    let mut tgt_bytes = [0u8; 10];
    let tgt_str = msg_format(&mut tgt_bytes, "G:", state.target_score);
    ui.label(rect_tgt, tgt_str).center().draw();

    let time_passed = state.level_start_time.elapsed().as_secs();
    let time_left = LEVEL_DURATION_SECS.saturating_sub(time_passed);
    let mut time_bytes = [0u8; 8];
    let time_str = msg_format(&mut time_bytes, "T:", time_left as u32);
    ui.label(rect_time, time_str).center().draw();

    let mut score_bytes = [0u8; 12];
    let score_str = msg_format(&mut score_bytes, "$", state.score);
    ui.label(rect_score, score_str).center().draw();

    ui.horizontal_divider(rect_divider);

    // Draw Miner Drilling Base Crane Pulley Block Structure
    ui.draw_filled_rect(Rect::new(60, 11, 9, 3), BinaryColor::On);

    // Compute Raycast Hook Vector Trajectory Points
    let end_x = (64.0 + state.hook_angle.sin() * state.hook_length) as i32;
    let end_y = (14.0 + state.hook_angle.cos() * state.hook_length) as i32;

    // Draw Cable Rope Core Lines using precise step-by-step pixel matrix iteration
    let steps = (state.hook_length as i32).max(1);
    for s in 0..steps {
        let lx = 64 + ((end_x - 64) * s / steps);
        let ly = 14 + ((end_y - 14) * s / steps);
        ui.draw_filled_rect(Rect::new(lx, ly, 1, 1), BinaryColor::On);
    }

    // Render Hook Head Cross Tip
    ui.draw_stroke_rect(Rect::new(end_x - 1, end_y - 1, 3, 3), BinaryColor::On, 1);

    // Render Environmental Mineral Distribution Layout Matrix Elements
    for mineral in &state.minerals {
        if mineral.active {
            let r = mineral.radius as i32;
            let mx = mineral.x as i32;
            let my = mineral.y as i32;

            if mineral.value >= 100 {
                // High-value assets (Gold & Diamonds) are rendered as filled circles
                // Draw the center core pixel
                ui.draw_filled_rect(Rect::new(mx, my, 1, 1), BinaryColor::On);

                // Fill out the rest of the circle using concentric rings
                for rad in 1..=r {
                    ui.draw_procedural_circle(mx, my, rad);
                }
            } else {
                // Stones are represented by outlined circular structures
                ui.draw_procedural_circle(mx, my, r);
            }
        }
    }

    // === Level Pass Screen Overlay Dialog ===
    if state.is_level_complete {
        ui.draw_filled_rect(rect_board, BinaryColor::Off);
        ui.draw_stroke_rect(rect_board, BinaryColor::On, 1);

        let mut rect_line1 = Rect::default();
        let mut rect_line2 = Rect::default();

        let pass_layout = FlexNode::new(FlexDirection::Column)
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

        pass_layout.layout(rect_board);

        ui.label(rect_line1, "LEVEL PASS").center().draw();
        ui.label(rect_line2, "[7] Next Level").center().draw();
    }

    // === Game Over Screen Overlay Dialog ===
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
