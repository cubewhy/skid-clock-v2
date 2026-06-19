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

const WHEEL_X: i32 = 64;
const WHEEL_Y: i32 = 30;
const WHEEL_RADIUS: i32 = 10;
const NEEDLE_LENGTH: i32 = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Playing,
    GameOver,
    GameWin,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PinNeedle {
    pub relative_angle: f32,
}

pub struct StickNeedleState {
    phase: GamePhase,
    needles: Vec<PinNeedle>,
    remaining_needles: u8,
    wheel_angle: f32,
    wheel_speed: f32,
    flying_y: f32,
    needle_flying: bool,
    first_frame: bool,
    fire_locked: bool,
    level: u16,
    last_update: Instant,
    show_hit_frame: bool,
}

impl Default for StickNeedleState {
    fn default() -> Self {
        let now = Instant::now();
        let mut state = Self {
            phase: GamePhase::Playing,
            needles: Vec::with_capacity(32),
            remaining_needles: 15,
            wheel_angle: 0.0,
            wheel_speed: 0.04,
            flying_y: 56.0,
            needle_flying: false,
            first_frame: true,
            fire_locked: false,
            level: 1,
            last_update: now,
            show_hit_frame: false,
        };
        state.init_pin_game();
        state
    }
}

impl StickNeedleState {
    fn init_pin_game(&mut self) {
        let now = Instant::now();
        self.needles.clear();

        // Dynamically compute targeted needle counts
        self.remaining_needles = 8 + if self.level > 8 { 8 } else { self.level as u8 };

        self.first_frame = true;
        self.fire_locked = false;
        self.wheel_angle = 0.0;

        // Compute base rotational configurations derived from level progressions
        let mut base_speed = 0.03 + (self.level as f32 * 0.002);
        if base_speed > 0.055 {
            base_speed = 0.055;
        }
        self.wheel_speed = base_speed;

        self.flying_y = 56.0;
        self.needle_flying = false;
        self.show_hit_frame = false;
        self.phase = GamePhase::Playing;
        self.last_update = now;

        // Generate pre-pinned procedural obstruction elements matching level index
        let mut pre_pinned_count = 2 + (self.level % 4) as u8;
        if pre_pinned_count + self.remaining_needles >= 32 {
            pre_pinned_count = 32 - self.remaining_needles - 1;
        }

        for i in 0..pre_pinned_count {
            let angle = (2.0 * core::f32::consts::PI / pre_pinned_count as f32) * i as f32;
            self.needles.push(PinNeedle {
                relative_angle: angle,
            });
        }
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut StickNeedleState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    let now = Instant::now();

    // Evaluate if user is interacting with structural directions or action keys
    let joy_pushed = ctx.input_manager.is_down(UiEvents::UP)
        || ctx.input_manager.is_down(UiEvents::DOWN)
        || ctx.input_manager.is_down(UiEvents::LEFT)
        || ctx.input_manager.is_down(UiEvents::RIGHT);

    let clicked = ctx.input_manager.is_down(UiEvents::KEY_7);

    if state.phase == GamePhase::GameOver || state.phase == GamePhase::GameWin {
        if !joy_pushed {
            state.fire_locked = false;
        }

        if clicked || (joy_pushed && !state.fire_locked) {
            if state.phase == GamePhase::GameWin {
                state.level += 1;
                state.init_pin_game();
            } else {
                state.level = 1;
                return Some(App::games_menu());
            }
            return None;
        }
        return None;
    }

    let mut dt = now.duration_since(state.last_update).as_secs_f32() * 1000.0 / 25.0;
    state.last_update = now;

    if dt > 3.0 {
        dt = 3.0;
    }

    // 1. Wheel Rotational Mechanics
    let mut current_base_speed = 0.03 + (state.level as f32 * 0.002);
    if current_base_speed > 0.055 {
        current_base_speed = 0.055;
    }

    let pinned_count = state.needles.len() as f32;
    let direction_modifier = if state.needles.len().is_multiple_of(5) {
        -1.0
    } else {
        1.0
    };
    state.wheel_speed = (current_base_speed + (pinned_count * 0.003)) * direction_modifier;
    state.wheel_angle += state.wheel_speed * dt;

    // Keep rotation bound cleanly within complete modular boundaries
    if state.wheel_angle > 2.0 * core::f32::consts::PI {
        state.wheel_angle -= 2.0 * core::f32::consts::PI;
    }
    if state.wheel_angle < 0.0 {
        state.wheel_angle += 2.0 * core::f32::consts::PI;
    }

    // 2. Input Firing Latches
    if state.first_frame {
        state.first_frame = false;
        state.fire_locked = joy_pushed;
    }

    if joy_pushed {
        if !state.fire_locked && !state.needle_flying {
            state.needle_flying = true;
            state.flying_y = 56.0;
            state.fire_locked = true;
        }
    } else {
        state.fire_locked = false;
    }

    // 3. Projectile Simulation & Angular Overlap Evaluation
    state.show_hit_frame = false;
    if state.needle_flying {
        state.flying_y -= 4.5 * dt;

        if state.flying_y <= (WHEEL_Y + WHEEL_RADIUS) as f32 {
            state.needle_flying = false;
            state.show_hit_frame = true;

            let mut hit_angle = core::f32::consts::FRAC_PI_2 - state.wheel_angle;
            while hit_angle < 0.0 {
                hit_angle += 2.0 * core::f32::consts::PI;
            }
            while hit_angle >= 2.0 * core::f32::consts::PI {
                hit_angle -= 2.0 * core::f32::consts::PI;
            }

            let mut collision = false;
            for needle in &state.needles {
                let mut diff = (hit_angle - needle.relative_angle).abs();
                if diff > core::f32::consts::PI {
                    diff = 2.0 * core::f32::consts::PI - diff;
                }
                // Collision threshold mapping directly to the original 0.22 radians
                if diff < 0.22 {
                    collision = true;
                    break;
                }
            }

            if collision {
                state.phase = GamePhase::GameOver;
                state.fire_locked = joy_pushed;
            } else {
                state.needles.push(PinNeedle {
                    relative_angle: hit_angle,
                });
                state.remaining_needles -= 1;
                if state.remaining_needles == 0 {
                    state.phase = GamePhase::GameWin;
                    state.fire_locked = joy_pushed;
                }
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &StickNeedleState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    if state.phase == GamePhase::GameOver || state.phase == GamePhase::GameWin {
        let mut rect_title = Rect::default();
        let mut rect_stats = Rect::default();
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
                    .assign_to(&mut rect_stats),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 14)
                    .assign_to(&mut rect_hint),
            );

        root.layout(display_bounds);

        match state.phase {
            GamePhase::GameWin => {
                ui.label(rect_title, "CLEAR!").center().draw();
                let mut bytes = [0u8; 24];
                let passed_str = win_format(&mut bytes, state.level);
                ui.label(rect_stats, passed_str).center().draw();
                ui.label(rect_hint, "[PRESS UP] NEXT").center().draw();
            }
            _ => {
                ui.label(rect_title, "GAME OVER").center().draw();
                let mut bytes = [0u8; 24];
                let failed_str = lose_format(&mut bytes, state.level);
                ui.label(rect_stats, failed_str).center().draw();
                ui.label(rect_hint, "[PRESS UP] EXIT").center().draw();
            }
        }
        return;
    }

    // Draw Header Interface
    let mut hud_bytes = [0u8; 32];
    let hud_str = hud_format(&mut hud_bytes, state.level, state.remaining_needles);
    ui.label(Rect::new(2, 0, 124, 10), hud_str).draw();
    ui.draw_line((0, 9), (128, 9), BinaryColor::On);

    // Draw Core Wheel Geometric Target Frame
    // Simulating circular coordinates with standard quadrant offsets
    for r in 0..=WHEEL_RADIUS {
        let d = ((WHEEL_RADIUS * WHEEL_RADIUS - r * r) as f32)
            .sqrt()
            .round() as i32;
        ui.draw_filled_rect(Rect::new(WHEEL_X - r, WHEEL_Y - d, 1, 1), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(WHEEL_X - r, WHEEL_Y + d, 1, 1), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(WHEEL_X + r, WHEEL_Y - d, 1, 1), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(WHEEL_X + r, WHEEL_Y + d, 1, 1), BinaryColor::On);
    }

    // Draw remaining counts cleanly inside the core circle segment
    let mut count_bytes = [0u8; 4];
    let count_str = number_format(&mut count_bytes, state.remaining_needles as u32);
    ui.label(Rect::new(WHEEL_X - 5, WHEEL_Y - 4, 12, 10), count_str)
        .draw();

    // 4. Render All Locked Target Needles Vector Extensions
    for needle in &state.needles {
        let current_abs_angle = state.wheel_angle + needle.relative_angle;
        let start_x = WHEEL_X + (current_abs_angle.cos() * WHEEL_RADIUS as f32).round() as i32;
        let start_y = WHEEL_Y + (current_abs_angle.sin() * WHEEL_RADIUS as f32).round() as i32;
        let end_x = WHEEL_X
            + (current_abs_angle.cos() * (WHEEL_RADIUS + NEEDLE_LENGTH) as f32).round() as i32;
        let end_y = WHEEL_Y
            + (current_abs_angle.sin() * (WHEEL_RADIUS + NEEDLE_LENGTH) as f32).round() as i32;

        ui.draw_line((start_x, start_y), (end_x, end_y), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(end_x, end_y, 1, 1), BinaryColor::On);
    }

    // 5. Draw Firing Sequences / Waiting Node Units
    if state.needle_flying {
        let fy = state.flying_y as i32;
        ui.draw_line(
            (WHEEL_X, fy),
            (WHEEL_X, fy + NEEDLE_LENGTH),
            BinaryColor::On,
        );
        ui.draw_filled_rect(
            Rect::new(WHEEL_X, fy + NEEDLE_LENGTH, 1, 1),
            BinaryColor::On,
        );
    } else if state.show_hit_frame {
        let hit_y = WHEEL_Y + WHEEL_RADIUS;
        ui.draw_line(
            (WHEEL_X, hit_y),
            (WHEEL_X, hit_y + NEEDLE_LENGTH),
            BinaryColor::On,
        );
        ui.draw_filled_rect(
            Rect::new(WHEEL_X, hit_y + NEEDLE_LENGTH, 1, 1),
            BinaryColor::On,
        );
    } else if state.remaining_needles > 0 {
        ui.draw_line(
            (WHEEL_X, 56),
            (WHEEL_X, 56 + NEEDLE_LENGTH),
            BinaryColor::On,
        );
        ui.draw_filled_rect(
            Rect::new(WHEEL_X, 56 + NEEDLE_LENGTH, 1, 1),
            BinaryColor::On,
        );
    }
}

fn hud_format(buf: &mut [u8], level: u16, remaining: u8) -> &str {
    let mut idx = 0;
    let lbl_lv = b"LV:";
    buf[idx..idx + lbl_lv.len()].copy_from_slice(lbl_lv);
    idx += lbl_lv.len();

    let mut temp = level;
    let lv_start = idx;
    if temp == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        while temp > 0 {
            buf[idx] = b'0' + (temp % 10) as u8;
            temp /= 10;
            idx += 1;
        }
        buf[lv_start..idx].reverse();
    }

    while idx < 18 {
        buf[idx] = b' ';
        idx += 1;
    }

    let lbl_rem = b"LEFT:";
    buf[idx..idx + lbl_rem.len()].copy_from_slice(lbl_rem);
    idx += lbl_rem.len();

    let rem_start = idx;
    let mut temp_rem = remaining;
    if temp_rem == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        while temp_rem > 0 {
            buf[idx] = b'0' + (temp_rem % 10);
            temp_rem /= 10;
            idx += 1;
        }
        buf[rem_start..idx].reverse();
    }

    core::str::from_utf8(&buf[..idx]).unwrap_or("LV:1 LEFT:0")
}

fn win_format(buf: &mut [u8], level: u16) -> &str {
    let mut idx = 0;
    let lbl = b"Level ";
    buf[idx..idx + lbl.len()].copy_from_slice(lbl);
    idx += lbl.len();

    let start = idx;
    let mut temp = level;
    while temp > 0 {
        buf[idx] = b'0' + (temp % 10) as u8;
        temp /= 10;
        idx += 1;
    }
    buf[start..idx].reverse();

    let suff = b" Passed!";
    buf[idx..idx + suff.len()].copy_from_slice(suff);
    idx += suff.len();

    core::str::from_utf8(&buf[..idx]).unwrap_or("Level Passed!")
}

fn lose_format(buf: &mut [u8], level: u16) -> &str {
    let mut idx = 0;
    let lbl = b"Reached Level: ";
    buf[idx..idx + lbl.len()].copy_from_slice(lbl);
    idx += lbl.len();

    let start = idx;
    let mut temp = level;
    while temp > 0 {
        buf[idx] = b'0' + (temp % 10) as u8;
        temp /= 10;
        idx += 1;
    }
    buf[start..idx].reverse();

    core::str::from_utf8(&buf[..idx]).unwrap_or("Reached Level: 1")
}

fn number_format(buf: &mut [u8], val: u32) -> &str {
    let mut idx = 0;
    let mut temp = val;
    if temp == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        while temp > 0 {
            buf[idx] = b'0' + (temp % 10) as u8;
            temp /= 10;
            idx += 1;
        }
        buf[0..idx].reverse();
    }
    core::str::from_utf8(&buf[..idx]).unwrap_or("0")
}
