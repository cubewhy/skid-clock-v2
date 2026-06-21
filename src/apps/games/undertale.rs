use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::UnifiedDisplay,
    ui::{
        Rect, Ui, UiEvents,
        layout::{AlignItems, FlexDirection, FlexNode},
    },
};
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};
use rand::{RngExt, rngs::SmallRng};
use std::sync::OnceLock;
use std::time::Instant;

const MAX_UT_BULLETS: usize = 16;
const BOX_X: i32 = 4;
const BOX_Y: i32 = 16;
const BOX_W: i32 = 68;
const BOX_H: i32 = 44;

#[derive(Debug, Clone, Copy)]
struct UTBullet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    bullet_type: u8, // 0: Karma particle, 1: Rising bottom bone, 2: Falling top bone
    param: f32,      // Bone height specification
    active: bool,
}

impl Default for UTBullet {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            bullet_type: 0,
            param: 0.0,
            active: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GasterBlaster {
    target_pos: i32,
    is_vertical: bool,
    timer: Instant,
    stage: u8, // 0: Inactive, 1: Locking warning ray, 2: Firing laser beam
    active: bool,
}

pub struct UndertaleState {
    bullets: [UTBullet; MAX_UT_BULLETS],
    blaster: GasterBlaster,
    soul_x: f32,
    soul_y: f32,
    hp: i32,
    max_hp: i32,
    wave: u8,
    wave_start_time: Instant,
    last_bullet_spawn: Instant,
    invul_frames: u8,
    is_game_over: bool,
    is_game_win: bool,
    last_tick: Instant,
    rng: SmallRng,
}

const UT_QUOTES: [[&str; 3]; 5] = [
    ["SANS:", "kids", "like you"],     // Wave 1
    ["should", "be", "burning."],      // Wave 2
    ["BadTime", "is", "coming."],      // Wave 3
    ["STAY", "DETER-", "MINED!"],      // Wave 4
    ["It's a", "beauti-", "ful day."], // Default Fallback
];

impl Default for UndertaleState {
    fn default() -> Self {
        let rng = rand::make_rng();

        Self {
            bullets: [UTBullet::default(); MAX_UT_BULLETS],
            blaster: GasterBlaster {
                target_pos: 0,
                is_vertical: false,
                timer: Instant::now(),
                stage: 0,
                active: false,
            },
            soul_x: (BOX_X + BOX_W / 2) as f32,
            soul_y: (BOX_Y + BOX_H / 2) as f32,
            hp: 92,
            max_hp: 92,
            wave: 1,
            wave_start_time: Instant::now(),
            last_bullet_spawn: Instant::now(),
            invul_frames: 0,
            is_game_over: false,
            is_game_win: false,
            last_tick: Instant::now(),
            rng,
        }
    }
}

impl UndertaleState {
    fn draw_soul_heart<D>(&self, ui: &mut Ui<'_, D>, x: i32, y: i32)
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        // Blink rendering handler mapping to active invincibility frames
        if self.invul_frames > 0 && (state_millis_now() / 80).is_multiple_of(2) {
            return;
        }
        ui.draw_filled_rect(Rect::new(x + 1, y, 1, 1), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(x + 3, y, 1, 1), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(x, y + 1, 5, 2), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(x + 1, y + 3, 3, 1), BinaryColor::On);
        ui.draw_filled_rect(Rect::new(x + 2, y + 4, 1, 1), BinaryColor::On);
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut UndertaleState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC)
        || (state.is_game_over || state.is_game_win) && ctx.menu_events.contains(UiEvents::CONFIRM)
    {
        return Some(App::games_menu());
    }

    if state.is_game_over || state.is_game_win {
        return None;
    }

    // Process kinematic steps scaling increments using standard 25ms base frame targets
    let elapsed = state.last_tick.elapsed().as_millis();
    if elapsed == 0 {
        return None;
    }
    state.last_tick = Instant::now();
    let mut dt = (elapsed as f32) / 25.0;
    if dt > 3.0 {
        dt = 3.0;
    }

    if state.invul_frames > 0 {
        state.invul_frames = state.invul_frames.saturating_sub(1);
    }

    // === 1. Player Soul Heart Translation Updates ===
    let soul_speed = 1.6 * dt;
    if ctx.input_manager.is_down(UiEvents::LEFT) {
        state.soul_x -= soul_speed;
    } else if ctx.input_manager.is_down(UiEvents::RIGHT) {
        state.soul_x += soul_speed;
    }
    if ctx.input_manager.is_down(UiEvents::UP) {
        state.soul_y -= soul_speed;
    } else if ctx.input_manager.is_down(UiEvents::DOWN) {
        state.soul_y += soul_speed;
    }

    // Enforce static boundary constraint zones clamping coordinates securely within the bounding area box
    state.soul_x = state
        .soul_x
        .clamp((BOX_X + 2) as f32, (BOX_X + BOX_W - 7) as f32);
    state.soul_y = state
        .soul_y
        .clamp((BOX_Y + 2) as f32, (BOX_Y + BOX_H - 7) as f32);

    // === 2. Wave Sequence Control Tracking Framework ===
    if state.wave_start_time.elapsed().as_millis() > 15000 {
        state.wave += 1;
        state.wave_start_time = Instant::now();

        // Wipe trailing patterns to reset arena transitions seamlessly
        for bullet in &mut state.bullets {
            bullet.active = false;
        }
        state.blaster.active = false;
        state.blaster.stage = 0;

        if state.wave > 4 {
            state.is_game_win = true;
            return None;
        }
    }

    // === 3. Dynamic Obstacle / Projectile Pattern Spawning Engine ===
    let base_rate = 800 - (state.wave as i32 * 120);
    let spawn_rate_ms = base_rate.clamp(250, 1000) as u128;

    if state.last_bullet_spawn.elapsed().as_millis() > spawn_rate_ms {
        state.last_bullet_spawn = Instant::now();

        // Spawning protocols for rising and falling bone wall segments
        if state.wave == 1 || state.wave == 3 {
            for i in 0..MAX_UT_BULLETS {
                if !state.bullets[i].active {
                    state.bullets[i].active = true;
                    state.bullets[i].bullet_type = state.rng.random_range(1..3) as u8;
                    state.bullets[i].x = (BOX_X + BOX_W - 2) as f32;
                    state.bullets[i].vx = -(1.2 + state.wave as f32 * 0.2);
                    state.bullets[i].vy = 0.0;
                    state.bullets[i].param = state.rng.random_range(12..24) as f32;

                    if state.bullets[i].bullet_type == 2 {
                        state.bullets[i].y = (BOX_Y + 1) as f32;
                    } else {
                        state.bullets[i].y = (BOX_Y + BOX_H - 1) as f32;
                    }
                    break;
                }
            }
        }

        // Spawning protocols for radial targeting karma particles trajectories
        if state.wave >= 2 {
            for i in 0..MAX_UT_BULLETS {
                if !state.bullets[i].active && state.rng.random_range(0..10) < 4 {
                    state.bullets[i].active = true;
                    state.bullets[i].bullet_type = 0;

                    let angle_deg = state.rng.random_range(0..360) as f32;
                    let angle_rad = angle_deg * std::f32::consts::PI / 180.0;

                    state.bullets[i].x = state.soul_x + angle_rad.cos() * 45.0;
                    state.bullets[i].y = state.soul_y + angle_rad.sin() * 45.0;
                    state.bullets[i].vx = -angle_rad.cos() * 1.3;
                    state.bullets[i].vy = -angle_rad.sin() * 1.3;
                    break;
                }
            }
        }
    }

    // Asynchronous triggering logic for charging a Gaster Blaster cannon
    if state.wave >= 3 && !state.blaster.active && state.rng.random_range(0..100) < 3 {
        state.blaster.active = true;
        state.blaster.stage = 1;
        state.blaster.timer = Instant::now();
        state.blaster.is_vertical = state.rng.random_range(0..2) == 0;
        state.blaster.target_pos = if state.blaster.is_vertical {
            state.rng.random_range((BOX_X + 10)..(BOX_X + BOX_W - 10))
        } else {
            state.rng.random_range((BOX_Y + 8)..(BOX_Y + BOX_H - 8))
        };
    }

    // === 4. Collision Resolution and Kinematic Steps ===
    if state.blaster.active {
        if state.blaster.stage == 1 && state.blaster.timer.elapsed().as_millis() > 700 {
            state.blaster.stage = 2;
            state.blaster.timer = Instant::now();
        } else if state.blaster.stage == 2 && state.blaster.timer.elapsed().as_millis() > 500 {
            state.blaster.active = false;
            state.blaster.stage = 0;
        }

        // Calculate laser ray intercept coordinates matching player bounding footprints
        if state.blaster.stage == 2 && state.invul_frames == 0 {
            let mut is_hit = false;
            if state.blaster.is_vertical {
                if state.soul_x + 2.0 >= (state.blaster.target_pos - 4) as f32
                    && state.soul_x <= (state.blaster.target_pos + 4) as f32
                {
                    is_hit = true;
                }
            } else {
                if state.soul_y + 2.0 >= (state.blaster.target_pos - 4) as f32
                    && state.soul_y <= (state.blaster.target_pos + 4) as f32
                {
                    is_hit = true;
                }
            }

            if is_hit {
                state.hp -= 14;
                state.invul_frames = 15;
                if state.hp <= 0 {
                    state.hp = 0;
                    state.is_game_over = true;
                }
            }
        }
    }

    // Evaluate standard bullet intersection loops criteria
    for i in 0..MAX_UT_BULLETS {
        if !state.bullets[i].active {
            continue;
        }

        state.bullets[i].x += state.bullets[i].vx * dt;
        state.bullets[i].y += state.bullets[i].vy * dt;

        // Dynamic edge destruction criteria thresholds
        if state.bullets[i].x < BOX_X as f32
            || state.bullets[i].x > (BOX_X + BOX_W) as f32
            || state.bullets[i].y < BOX_Y as f32
            || state.bullets[i].y > (BOX_Y + BOX_H) as f32
        {
            state.bullets[i].active = false;
            continue;
        }

        if state.invul_frames == 0 {
            let mut collision = false;

            if state.bullets[i].bullet_type == 0 {
                if (state.bullets[i].x - (state.soul_x + 2.0)).abs() < 4.0
                    && (state.bullets[i].y - (state.soul_y + 2.0)).abs() < 4.0
                {
                    collision = true;
                }
            } else if state.bullets[i].bullet_type == 1 {
                if state.bullets[i].x >= state.soul_x - 1.0
                    && state.bullets[i].x <= state.soul_x + 5.0
                    && state.soul_y + 4.0 >= (BOX_Y + BOX_H) as f32 - state.bullets[i].param
                {
                    collision = true;
                }
            } else if state.bullets[i].bullet_type == 2
                && state.bullets[i].x >= state.soul_x - 1.0
                && state.bullets[i].x <= state.soul_x + 5.0
                && state.soul_y <= BOX_Y as f32 + state.bullets[i].param
            {
                collision = true;
            }

            if collision {
                state.hp -= 6;
                state.invul_frames = 12;
                if state.hp <= 0 {
                    state.hp = 0;
                    state.is_game_over = true;
                }
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &UndertaleState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    // === Game Defeat / Terminal Victory Overlay Interceptors ===
    if state.is_game_over || state.is_game_win {
        let mut rect_msg1 = Rect::default();
        let mut rect_msg2 = Rect::default();
        let mut rect_msg3 = Rect::default();

        let info_layout = FlexNode::new(FlexDirection::Column)
            .align_items(AlignItems::Stretch)
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 22)
                    .assign_to(&mut rect_msg1),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_size(display_bounds.width, 16)
                    .assign_to(&mut rect_msg2),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_flex(1)
                    .assign_to(&mut rect_msg3),
            );

        info_layout.layout(display_bounds);

        if state.is_game_win {
            ui.label(rect_msg1, "YOU WIN!").draw();
            ui.label(rect_msg2, "You saved the timeline.").draw();
        } else {
            ui.label(rect_msg1, "GAME OVER").draw();
            ui.label(rect_msg2, "Don't lose determination!").draw();
        }
        ui.label(rect_msg3, "[Esc] Exit Menu").draw();
        return;
    }

    // Main interface parameters setup pipelines layout splitting
    let mut rect_top_bar = Rect::default();
    let mut rect_arena_split = Rect::default();

    let root = FlexNode::new(FlexDirection::Column)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 10)
                .assign_to(&mut rect_top_bar),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut rect_arena_split),
        );

    root.layout(display_bounds);

    // Render Stats Headers Area Data Elements
    let mut hp_lbl_bytes = [0u8; 16];
    let hp_str = msg_format(&mut hp_lbl_bytes, "LV19  HP ", state.hp as u32);
    ui.label(Rect::new(2, 0, 56, 9), hp_str).draw();

    // Health Status Bar Graphic Elements
    ui.draw_stroke_rect(Rect::new(70, 1, 20, 6), BinaryColor::On, 1);
    let mapped_fill_w = ((state.hp as f32 / state.max_hp as f32) * 18.0).clamp(0.0, 18.0) as u32;
    if mapped_fill_w > 0 {
        ui.draw_filled_rect(Rect::new(71, 2, mapped_fill_w, 4), BinaryColor::On);
    }

    let mut wave_lbl_bytes = [0u8; 8];
    let wave_str = msg_format(&mut wave_lbl_bytes, "W:", state.wave as u32);
    ui.label(Rect::new(104, 0, 24, 9), wave_str).draw();

    ui.draw_filled_rect(Rect::new(0, 9, 128, 1), BinaryColor::On);

    // Render Central Arena Boundary Box Container Framework
    ui.draw_stroke_rect(
        Rect::new(BOX_X, BOX_Y, BOX_W as u32, BOX_H as u32),
        BinaryColor::On,
        1,
    );

    // Right Content Panel Side Dialogue Interface Processing Elements
    ui.label(Rect::new(76, 14, 50, 9), "* Sans").draw();
    ui.draw_filled_rect(Rect::new(76, 23, 48, 1), BinaryColor::On);

    let quote_idx = if state.wave <= 4 {
        (state.wave - 1) as usize
    } else {
        4
    };

    for (line, &quote_text) in UT_QUOTES[quote_idx].iter().take(3).enumerate() {
        ui.label(Rect::new(76, 26 + (line as i32 * 9), 52, 9), quote_text)
            .draw();
    }

    // === Render Gaster Blaster Cannons Warn / Laser Lines Processing ===
    if state.blaster.active {
        if state.blaster.stage == 1 {
            if (state_millis_now() / 50).is_multiple_of(2) {
                if state.blaster.is_vertical {
                    ui.draw_filled_rect(
                        Rect::new(state.blaster.target_pos, BOX_Y + 1, 1, (BOX_H - 2) as u32),
                        BinaryColor::On,
                    );
                } else {
                    ui.draw_filled_rect(
                        Rect::new(BOX_X + 1, state.blaster.target_pos, (BOX_W - 2) as u32, 1),
                        BinaryColor::On,
                    );
                }
            }
        } else if state.blaster.stage == 2 {
            if state.blaster.is_vertical {
                ui.draw_filled_rect(
                    Rect::new(
                        state.blaster.target_pos - 4,
                        BOX_Y + 1,
                        9,
                        (BOX_H - 2) as u32,
                    ),
                    BinaryColor::On,
                );
            } else {
                ui.draw_filled_rect(
                    Rect::new(
                        BOX_X + 1,
                        state.blaster.target_pos - 4,
                        (BOX_W - 2) as u32,
                        9,
                    ),
                    BinaryColor::On,
                );
            }
        }
    }

    // === Render Active Projectile and Bone Matrix Elements ===
    for bullet in &state.bullets {
        if !bullet.active {
            continue;
        }

        let bx = bullet.x as i32;
        let by = bullet.y as i32;

        if bullet.bullet_type == 0 {
            // Karma Core Particles Node Clusters Map Elements
            ui.draw_filled_rect(Rect::new(bx, by, 1, 1), BinaryColor::On);
            ui.draw_filled_rect(Rect::new(bx - 1, by, 3, 1), BinaryColor::On);
            ui.draw_filled_rect(Rect::new(bx, by - 1, 1, 3), BinaryColor::On);
        } else if bullet.bullet_type == 1 {
            // Rising Bone segment graphics configuration parameters layouts
            let h = bullet.param as i32;
            let bottom_y = BOX_Y + BOX_H - 1;
            ui.draw_filled_rect(Rect::new(bx, bottom_y - h, 1, h as u32), BinaryColor::On);
            ui.draw_filled_rect(
                Rect::new(bx + 2, bottom_y - h, 1, h as u32),
                BinaryColor::On,
            );
            ui.draw_filled_rect(Rect::new(bx - 1, bottom_y - h, 5, 1), BinaryColor::On);
        } else if bullet.bullet_type == 2 {
            // Falling Bone segment graphics configuration parameters layouts
            let h = bullet.param as i32;
            let top_y = BOX_Y + 1;
            ui.draw_filled_rect(Rect::new(bx, top_y, 1, h as u32), BinaryColor::On);
            ui.draw_filled_rect(Rect::new(bx + 2, top_y, 1, h as u32), BinaryColor::On);
            ui.draw_filled_rect(Rect::new(bx - 1, top_y + h, 5, 1), BinaryColor::On);
        }
    }

    // Top Layer Draw Overlay Target Entity (SOUL Heart character shape)
    state.draw_soul_heart(&mut ui, state.soul_x as i32, state.soul_y as i32);
}

// Utility safe system clock mapping configuration tracker
fn state_millis_now() -> u128 {
    static START_TIME: OnceLock<Instant> = OnceLock::new();
    START_TIME.get_or_init(Instant::now).elapsed().as_millis()
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
