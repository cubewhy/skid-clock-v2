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

const GRAVITY: f32 = 0.22;
const FLAP_FORCE: f32 = -2.3;
const PIPE_GAP_H: i16 = 22;
const PIPE_W: i16 = 10;
const BIRD_X: i32 = 25;

pub struct FlappyBirdState {
    bird_y: f32,
    bird_vel: f32,
    pipe_x: f32,
    pipe_gap_y: i16,
    score: u32,
    is_game_over: bool,
    joy_latch: bool,
    last_tick: Instant,
    rng: SmallRng,
}

impl Default for FlappyBirdState {
    fn default() -> Self {
        let mut rng: SmallRng = rand::make_rng();
        let initial_pipe_gap_y = rng.random_range(14..38) as i16;

        Self {
            bird_y: 32.0,
            bird_vel: 0.0,
            pipe_x: 128.0,
            pipe_gap_y: initial_pipe_gap_y,
            score: 0,
            is_game_over: false,
            joy_latch: false,
            last_tick: Instant::now(),
            rng,
        }
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut FlappyBirdState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    if state.is_game_over {
        if ctx.menu_events.contains(UiEvents::KEY_7) || ctx.menu_events.contains(UiEvents::CONFIRM)
        {
            *state = FlappyBirdState::default();
        }
        return None;
    }

    // Compute delta time frames relative to 25ms engine target ticks
    let elapsed = state.last_tick.elapsed().as_millis();
    if elapsed == 0 {
        return None;
    }
    state.last_tick = Instant::now();

    let mut dt = (elapsed as f32) / 25.0;
    if dt > 3.0 {
        dt = 3.0;
    }

    let flap = ctx.input_manager.is_down(UiEvents::all());

    // Single-flap impulse trigger via an edge-detect latch mechanism
    if flap {
        if !state.joy_latch {
            state.bird_vel = FLAP_FORCE;
            state.joy_latch = true;
        }
    } else {
        state.joy_latch = false;
    }

    // Apply gravity environment and physical kinematic transformations
    state.bird_vel += GRAVITY * dt;
    state.bird_y += state.bird_vel * dt;

    // Advance background scrolling pipes horizontally
    state.pipe_x -= 1.6 * dt;
    if state.pipe_x < -(PIPE_W as f32) {
        state.pipe_x = 128.0;
        state.pipe_gap_y = state.rng.random_range(14..38) as i16;
        state.score += 1;
    }

    // Ceiling and floor boundary collisions
    if state.bird_y < 10.0 || state.bird_y > 63.0 {
        state.is_game_over = true;
    }

    // AABB Bounding Box collision scan intersection checks
    let pipe_x_int = state.pipe_x as i32;
    let bird_y_int = state.bird_y as i32;

    if BIRD_X + 2 >= pipe_x_int
        && BIRD_X - 2 <= (pipe_x_int + PIPE_W as i32)
        && (bird_y_int - 2 < state.pipe_gap_y as i32
            || bird_y_int + 2 > (state.pipe_gap_y + PIPE_GAP_H) as i32)
    {
        state.is_game_over = true;
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &FlappyBirdState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut rect_score = Rect::default();
    let mut rect_divider = Rect::default();
    let mut rect_board = Rect::default();

    let root = FlexNode::new(FlexDirection::Column)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 11)
                .assign_to(&mut rect_score),
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

    // Render Score Statistics Header
    let mut score_bytes = [0u8; 16];
    let score_str = msg_format(&mut score_bytes, "FLAPPY: ", state.score);
    ui.label(rect_score, score_str).center().draw();
    ui.horizontal_divider(rect_divider);

    // Draw Symmetrical Upper and Lower Obstacle Pipes
    let px = state.pipe_x as i32;

    // Upper Pipe Segment
    if px < 128 {
        let upper_height = (state.pipe_gap_y - 10).max(0) as u32;
        ui.draw_filled_rect(
            Rect::new(px, 10, PIPE_W as u32, upper_height),
            BinaryColor::On,
        );
    }

    // Lower Pipe Segment
    if px < 128 {
        let lower_start_y = (state.pipe_gap_y + PIPE_GAP_H) as i32;
        let lower_height = (64 - lower_start_y).max(0) as u32;
        ui.draw_filled_rect(
            Rect::new(px, lower_start_y, PIPE_W as u32, lower_height),
            BinaryColor::On,
        );
    }

    // Render Avian Avatar Player Entity Shape (5x5 Cross Mask Alignment)
    let iby = state.bird_y as i32;
    ui.draw_filled_rect(Rect::new(BIRD_X - 2, iby, 5, 1), BinaryColor::On);
    ui.draw_filled_rect(Rect::new(BIRD_X, iby - 2, 1, 5), BinaryColor::On);
    ui.draw_filled_rect(Rect::new(BIRD_X - 1, iby - 1, 3, 3), BinaryColor::On);
    // Isolate black dot point matrix pixel layout array overlay coordinates for eye placement
    ui.draw_filled_rect(Rect::new(BIRD_X + 1, iby, 1, 1), BinaryColor::Off);

    // Game Over Overlay Modal Frame Mask Layout Window
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

        ui.label(rect_line1, "FLAP OVER").center().draw();
        ui.label(rect_line2, "[7] Replay").center().draw();
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
