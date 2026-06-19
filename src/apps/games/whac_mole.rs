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

#[derive(Debug, Clone, Copy)]
pub struct MoleHole {
    pub x: i16,
    pub y: i16,
}

const HOLES: [MoleHole; 8] = [
    MoleHole { x: 34, y: 20 },
    MoleHole { x: 64, y: 18 },
    MoleHole { x: 94, y: 20 }, // 0, 1, 2
    MoleHole { x: 30, y: 38 },
    MoleHole { x: 98, y: 38 }, // 3, 4
    MoleHole { x: 34, y: 56 },
    MoleHole { x: 64, y: 58 },
    MoleHole { x: 94, y: 56 }, // 5, 6, 7
];

pub struct WhacState {
    phase: GamePhase,
    active_mole_hole: i8,
    mole_spawn_time: u32,
    mole_duration: u32,
    whac_score: i32,
    whac_lives: i32,
    last_joy_dir: i8,
    player_hammer_pos: i8,
    mole_empty_start_time: u32,
    mole_empty_duration: u32,
    is_waiting_for_mole: bool,
    last_update: Instant,
    base_time: Instant,
    rng_seed: u32,
}

impl Default for WhacState {
    fn default() -> Self {
        let now = Instant::now();
        let mut state = Self {
            phase: GamePhase::Playing,
            active_mole_hole: -1,
            mole_spawn_time: 0,
            mole_duration: 1200,
            whac_score: 0,
            whac_lives: 3,
            last_joy_dir: -1,
            player_hammer_pos: -1,
            mole_empty_start_time: 0,
            mole_empty_duration: 200,
            is_waiting_for_mole: false,
            last_update: now,
            base_time: now,
            rng_seed: 98765,
        };
        state.init_whac_game();
        state
    }
}

impl WhacState {
    // Linear Congruential Generator matching runtime pseudo-random generation rules
    fn pseudo_rand(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        self.rng_seed = self.rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let rand_val = ((self.rng_seed / 65536) % 32768) as i32;
        min + (rand_val % (max - min))
    }

    fn init_whac_game(&mut self) {
        let now = Instant::now();
        self.whac_score = 0;
        self.whac_lives = 3;
        self.phase = GamePhase::Playing;
        self.last_joy_dir = -1;
        self.active_mole_hole = -1;
        self.player_hammer_pos = -1;
        self.mole_duration = 1200;
        self.is_waiting_for_mole = false;
        self.last_update = now;
        self.base_time = now;
    }

    fn get_whac_direction(&self, ctx: &UpdateContext) -> i8 {
        let up = ctx.input_manager.is_down(UiEvents::UP);
        let down = ctx.input_manager.is_down(UiEvents::DOWN);
        let left = ctx.input_manager.is_down(UiEvents::LEFT);
        let right = ctx.input_manager.is_down(UiEvents::RIGHT);

        if up && left {
            return 0;
        }
        if up && right {
            return 2;
        }
        if down && left {
            return 5;
        }
        if down && right {
            return 7;
        }
        if up {
            return 1;
        }
        if left {
            return 3;
        }
        if right {
            return 4;
        }
        if down {
            return 6;
        }

        -1
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut WhacState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    let now = Instant::now();
    let now_ms = now.duration_since(state.base_time).as_millis() as u32;
    state.last_update = now;

    if state.phase == GamePhase::GameOver {
        if ctx.input_manager.is_down(UiEvents::CONFIRM) || ctx.input_manager.is_down(UiEvents::UP) {
            return Some(App::games_menu());
        }
        return None;
    }

    let current_dir = state.get_whac_direction(ctx);
    state.player_hammer_pos = current_dir;

    // 1. Mole Lifecycle Processing Machine
    if state.active_mole_hole == -1 {
        if !state.is_waiting_for_mole {
            // Trigger safety intermediate latency buffer period
            state.is_waiting_for_mole = true;
            state.mole_empty_start_time = now_ms;
            state.mole_empty_duration = state.pseudo_rand(200, 500) as u32;
        }

        if now_ms.saturating_sub(state.mole_empty_start_time) > state.mole_empty_duration {
            // Transition out of buffer state, cleanly spawn active mole item
            state.active_mole_hole = state.pseudo_rand(0, 8) as i8;
            state.mole_spawn_time = now_ms;

            let mut dur = 1200 - (state.whac_score * 40);
            if dur < 450 {
                dur = 450;
            }
            state.mole_duration = dur as u32;
            state.is_waiting_for_mole = false;
        }
    } else {
        // Evaluate timeout expiration to reduce live tokens
        if now_ms.saturating_sub(state.mole_spawn_time) > state.mole_duration {
            state.active_mole_hole = -1;
            state.whac_lives -= 1;
            if state.whac_lives <= 0 {
                state.phase = GamePhase::GameOver;
                return None;
            }
        }
    }

    // 2. Edge-Triggered Hammer Strike Evaluation Logic
    if current_dir != -1
        && current_dir != state.last_joy_dir
        && current_dir == state.active_mole_hole
    {
        state.whac_score += 1;
        state.active_mole_hole = -1; // Despawn instantly into fallback window
    }
    state.last_joy_dir = current_dir;

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &WhacState) {
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
        let score_str = score_format(&mut score_bytes, "Final Score: ", state.whac_score);
        ui.label(rect_score, score_str).center().draw();
        ui.label(rect_hint, "[PRESS CONFIRM] RETURN")
            .center()
            .draw();
        return;
    }

    // Render Metrics HUD Status Line Elements
    let mut hud_bytes = [0u8; 32];
    let hud_str = hud_format(&mut hud_bytes, state.whac_score, state.whac_lives);
    ui.label(Rect::new(2, 0, 124, 10), hud_str).draw();
    ui.draw_line((0, 9), (128, 9), BinaryColor::On);

    // Render 8 Layout Mound Boundaries & Targets
    for (i, hole) in HOLES.iter().enumerate() {
        let hx = hole.x as i32;
        let hy = hole.y as i32;

        // Draw mound container frames procedurally (Simulating 16x5 rounded rects)
        ui.draw_filled_rect(Rect::new(hx - 7, hy - 2, 14, 1), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(hx - 7, hy + 2, 14, 1), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(hx - 8, hy - 1, 1, 3), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(hx + 7, hy - 1, 1, 3), BinaryColor::On);

        // Draw Active Mole Sprite Elements
        if i as i8 == state.active_mole_hole {
            ui.draw_procedural_circle(hx, hy - 4, 4);
            ui.draw_filled_rect(Rect::new(hx - 2, hy - 2, 5, 3), BinaryColor::On);
            // Reverse eye dot configurations inside the mole matrix
            ui.draw_filled_rect(Rect::new(hx - 1, hy - 5, 1, 1), BinaryColor::Off);
            ui.draw_filled_rect(Rect::new(hx + 1, hy - 5, 1, 1), BinaryColor::Off);
        }

        // Draw Target Selector / Crosshair Overlays
        if i as i8 == state.player_hammer_pos {
            ui.draw_procedural_circle(hx, hy - 3, 7);
            ui.draw_line((hx - 9, hy - 3), (hx - 7, hy - 3), BinaryColor::On);
            ui.draw_line((hx + 7, hy - 3), (hx + 9, hy - 3), BinaryColor::On);
        }
    }

    // Decorative Center Anchor Dot
    ui.draw_procedural_circle(64, 38, 2);
}

fn hud_format(buf: &mut [u8], score: i32, lives: i32) -> &str {
    let mut idx = 0;
    let lbl_title = b"WHAC-A-MOLE";
    buf[idx..idx + lbl_title.len()].copy_from_slice(lbl_title);
    idx += lbl_title.len();

    while idx < 15 {
        buf[idx] = b' ';
        idx += 1;
    }

    let lbl_s = b"S:";
    buf[idx..idx + lbl_s.len()].copy_from_slice(lbl_s);
    idx += lbl_s.len();

    let s_start = idx;
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
        buf[s_start..idx].reverse();
    }

    buf[idx] = b' ';
    idx += 1;

    let lbl_l = b"L:";
    buf[idx..idx + lbl_l.len()].copy_from_slice(lbl_l);
    idx += lbl_l.len();

    let l_start = idx;
    let mut temp_l = lives.max(0);
    if temp_l == 0 {
        buf[idx] = b'0';
        idx += 1;
    } else {
        while temp_l > 0 {
            buf[idx] = b'0' + (temp_l % 10) as u8;
            temp_l /= 10;
            idx += 1;
        }
        buf[l_start..idx].reverse();
    }

    core::str::from_utf8(&buf[..idx]).unwrap_or("WHAC-A-MOLE S:0 L:0")
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
