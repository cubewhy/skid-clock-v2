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

const SCREEN_WIDTH: u32 = 128;
const STACK_BLOCK_HEIGHT: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Playing,
    GameOver,
}

pub struct StackState {
    phase: GamePhase,
    stack_block_x: f32,
    stack_block_width: i32,
    stack_block_speed_x: f32,
    stack_is_falling: bool,
    stack_block_y: f32,
    tower_layer_count: i32,
    camera_view_offset_y: i32,
    joy_move_latched: bool,
    stack_entry_released: bool,
    tower_widths: Vec<i32>,
    tower_xs: Vec<i32>,
    last_update: Instant,
    base_time: Instant,
    accumulator: f32,
}

impl Default for StackState {
    fn default() -> Self {
        let now = Instant::now();
        let initial_width = 50;
        let initial_x = (SCREEN_WIDTH as i32 - initial_width) / 2;

        let mut state = Self {
            phase: GamePhase::Playing,
            stack_block_x: 0.0,
            stack_block_width: 45,
            stack_block_speed_x: 1.4,
            stack_is_falling: false,
            stack_block_y: 12.0,
            tower_layer_count: 1,
            camera_view_offset_y: 0,
            joy_move_latched: false,
            stack_entry_released: false,
            tower_widths: Vec::new(),
            tower_xs: Vec::new(),
            last_update: now,
            base_time: now,
            accumulator: 0.0,
        };

        state.tower_widths.push(initial_width);
        state.tower_xs.push(initial_x);
        state.stack_block_x = ((SCREEN_WIDTH as i32 - state.stack_block_width) / 2) as f32;
        state
    }
}

impl StackState {
    fn reset_game(&mut self) {
        let now = Instant::now();
        self.tower_layer_count = 1;
        self.phase = GamePhase::Playing;
        self.stack_block_width = 45;
        self.stack_block_x = ((SCREEN_WIDTH as i32 - self.stack_block_width) / 2) as f32;
        self.stack_block_y = 12.0;
        self.stack_block_speed_x = 1.4;
        self.stack_is_falling = false;
        self.camera_view_offset_y = 0;
        self.joy_move_latched = false;
        self.stack_entry_released = false;
        self.accumulator = 0.0;
        self.last_update = now;
        self.base_time = now;

        self.tower_widths.clear();
        self.tower_xs.clear();
        self.tower_widths.push(50);
        self.tower_xs.push((SCREEN_WIDTH as i32 - 50) / 2);
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut StackState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    if state.phase == GamePhase::GameOver {
        if ctx.input_manager.is_down(UiEvents::KEY_7) || ctx.input_manager.is_down(UiEvents::UP) {
            state.reset_game();
        }
        return None;
    }

    let now = Instant::now();
    let dt = now.duration_since(state.last_update).as_secs_f32();
    state.last_update = now;

    // Check raw input states matching structural joystick deviation flags
    let joy_moved = ctx.input_manager.is_down(UiEvents::UP)
        || ctx.input_manager.is_down(UiEvents::DOWN)
        || ctx.input_manager.is_down(UiEvents::LEFT)
        || ctx.input_manager.is_down(UiEvents::RIGHT);

    let clicked = ctx.input_manager.is_down(UiEvents::KEY_7);

    let mut processed_joy_moved = joy_moved;
    let mut processed_clicked = clicked;

    if !state.stack_entry_released {
        if !joy_moved && !clicked {
            state.stack_entry_released = true;
        }
        processed_joy_moved = false;
        processed_clicked = false;
    }

    if processed_joy_moved || processed_clicked {
        if !state.joy_move_latched && !state.stack_is_falling {
            state.stack_is_falling = true;
            state.joy_move_latched = true;
        }
    } else {
        state.joy_move_latched = false;
    }

    // Process gameplay ticks mapped to the exact 25ms frequency loop structure
    const TICK_INTERVAL: f32 = 0.025;
    state.accumulator += dt;

    while state.accumulator >= TICK_INTERVAL {
        state.accumulator -= TICK_INTERVAL;

        if !state.stack_is_falling {
            state.stack_block_x += state.stack_block_speed_x;

            if state.stack_block_x <= 0.0 && state.stack_block_speed_x < 0.0 {
                state.stack_block_x = 0.0;
                state.stack_block_speed_x = -state.stack_block_speed_x;
            } else if state.stack_block_x + state.stack_block_width as f32 >= SCREEN_WIDTH as f32
                && state.stack_block_speed_x > 0.0
            {
                state.stack_block_x = (SCREEN_WIDTH as i32 - state.stack_block_width) as f32;
                state.stack_block_speed_x = -state.stack_block_speed_x;
            }
        } else {
            state.stack_block_y += 3.5;
            let target_top_y = 58 - (state.tower_layer_count * STACK_BLOCK_HEIGHT as i32)
                + state.camera_view_offset_y;

            if state.stack_block_y >= target_top_y as f32 {
                state.stack_block_y = target_top_y as f32;
                state.stack_is_falling = false;

                let base_left = state.tower_xs[(state.tower_layer_count - 1) as usize];
                let base_right =
                    base_left + state.tower_widths[(state.tower_layer_count - 1) as usize];

                let cur_left = state.stack_block_x as i32;
                let cur_right = cur_left + state.stack_block_width;

                let overlap_left = base_left.max(cur_left);
                let overlap_right = base_right.min(cur_right);

                if overlap_right > overlap_left {
                    // Push overlapping block stats to dynamic collections (infinite capacity)
                    state.tower_xs.push(overlap_left);
                    state.tower_widths.push(overlap_right - overlap_left);

                    state.stack_block_width = overlap_right - overlap_left;
                    state.tower_layer_count += 1;

                    let current_highest_y = 58
                        - (state.tower_layer_count * STACK_BLOCK_HEIGHT as i32)
                        + state.camera_view_offset_y;
                    if current_highest_y < 24 {
                        state.camera_view_offset_y += STACK_BLOCK_HEIGHT as i32;
                    }

                    // Pseudo-random spawn offset derived safely from layout properties
                    let now_ms = now.duration_since(state.base_time).as_millis() as i32;
                    let max_rand_range = SCREEN_WIDTH as i32 - state.stack_block_width;
                    let rand_offset = if max_rand_range > 0 {
                        (now_ms ^ state.tower_layer_count) % max_rand_range
                    } else {
                        0
                    };

                    state.stack_block_x = rand_offset as f32;
                    state.stack_block_y = 12.0;

                    let direction_sign = if state.stack_block_speed_x > 0.0 {
                        1.0
                    } else {
                        -1.0
                    };
                    state.stack_block_speed_x =
                        direction_sign * (1.2 + (state.tower_layer_count as f32 * 0.12));

                    if state.stack_block_speed_x.abs() > 4.5 {
                        state.stack_block_speed_x = direction_sign * 4.5;
                    }
                } else {
                    state.phase = GamePhase::GameOver;
                }
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &StackState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

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

        let mut score_bytes = [0u8; 24];
        let score_str = results_format(&mut score_bytes, state.tower_layer_count - 1);
        ui.label(rect_score, score_str).center().draw();
        ui.label(rect_hint, "[PRESS UP TO RETRY]").center().draw();
        return;
    }

    // Render HUD display layout matching the original headers
    let mut hud_bytes = [0u8; 32];
    let hud_str = hud_format(&mut hud_bytes, state.tower_layer_count - 1);
    ui.label(Rect::new(2, 0, 124, 10), hud_str).draw();

    // Top horizontal interface threshold barrier line
    ui.draw_line((0, 9), (SCREEN_WIDTH as i32, 9), BinaryColor::On);

    // Draw solid tower block segments inside active vertical viewport bounds
    for i in 0..state.tower_layer_count {
        let draw_y = 58 - ((i + 1) * STACK_BLOCK_HEIGHT as i32) + state.camera_view_offset_y;
        if !(10..=64).contains(&draw_y) {
            continue;
        }
        ui.draw_filled_rect(
            Rect::new(
                state.tower_xs[i as usize],
                draw_y,
                state.tower_widths[i as usize] as u32,
                STACK_BLOCK_HEIGHT - 1,
            ),
            BinaryColor::On,
        );
    }

    // Draw outline frame container around the active dropping layer piece
    ui.draw_stroke_rect(
        Rect::new(
            state.stack_block_x as i32,
            state.stack_block_y as i32,
            state.stack_block_width as u32,
            STACK_BLOCK_HEIGHT - 1,
        ),
        BinaryColor::On,
        1,
    );
}

fn hud_format(buf: &mut [u8], layers: i32) -> &str {
    let mut idx = 0;

    let label_stack = b"STACK: ";
    buf[idx..idx + label_stack.len()].copy_from_slice(label_stack);
    idx += label_stack.len();

    let layers_start = idx;
    let mut temp = layers.max(0);
    if temp == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        while temp > 0 {
            buf[idx] = b'0' + (temp % 10) as u8;
            temp /= 10;
            idx += 1;
        }
        buf[layers_start..idx].reverse();
    }

    core::str::from_utf8(&buf[..idx]).unwrap_or("STACK: 0")
}

fn results_format(buf: &mut [u8], layers: i32) -> &str {
    let mut idx = 0;
    let label = b"Layers: ";
    buf[idx..idx + label.len()].copy_from_slice(label);
    idx += label.len();

    let start = idx;
    let mut temp = layers.max(0);
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

    core::str::from_utf8(&buf[..idx]).unwrap_or("Layers: 0")
}
