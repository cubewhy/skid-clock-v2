use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::UnifiedDisplay,
    ui::{
        Rect, Ui,
        ctx::UiEvents,
        layout::{FlexDirection, FlexNode},
    },
};
use chrono::Local;
use std::time::{Duration, Instant};

const HEADER_HEIGHT: u32 = 15;
const TOP_DIVIDER_HEIGHT: u32 = 2;
const BOTTOM_DIVIDER_HEIGHT: u32 = 1;
const FOOTER_HEIGHT: u32 = 14;

pub struct CountdownState {
    pub tick: u32,
    pub target_seconds: u32,
    pub elapsed: Duration,
    pub start_time: Option<Instant>,
    pub is_editing: bool,
    pub edit_field: u8, // 0: Minutes, 1: Seconds
}

impl Default for CountdownState {
    fn default() -> Self {
        Self {
            tick: 0,
            target_seconds: 60, // 1 Minute default
            elapsed: Duration::ZERO,
            start_time: None,
            is_editing: true,
            edit_field: 0,
        }
    }
}

pub fn update(ctx: &UpdateContext, state: &mut CountdownState) -> Option<App> {
    state.tick += 1;
    let events = ctx.menu_events;

    // Automatically pause/stop the timer if target duration is reached
    if let Some(start) = state.start_time {
        let total_elapsed = state.elapsed + start.elapsed();
        if total_elapsed >= Duration::from_secs(state.target_seconds as u64) {
            state.elapsed = Duration::from_secs(state.target_seconds as u64);
            state.start_time = None; // Countdown finished
        }
    }

    if state.is_editing {
        if events.contains(UiEvents::UP) {
            if state.edit_field == 0 {
                state.target_seconds = state.target_seconds.saturating_add(60).min(3599);
            } else {
                state.target_seconds =
                    (state.target_seconds / 60) * 60 + ((state.target_seconds % 60) + 1) % 60;
            }
        }

        if events.contains(UiEvents::DOWN) {
            if state.edit_field == 0 {
                state.target_seconds = state.target_seconds.saturating_sub(60);
            } else {
                state.target_seconds =
                    (state.target_seconds / 60) * 60 + ((state.target_seconds % 60) + 59) % 60;
            }
        }

        if events.intersects(UiEvents::LEFT | UiEvents::RIGHT) {
            state.edit_field = if state.edit_field == 0 { 1 } else { 0 };
        }

        if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) && state.target_seconds > 0 {
            state.is_editing = false;
            state.elapsed = Duration::ZERO;
            state.start_time = Some(Instant::now());
        }
    } else {
        // Dynamic control while running/paused
        if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) {
            let total_elapsed = match state.start_time {
                Some(start) => state.elapsed + start.elapsed(),
                None => state.elapsed,
            };

            // Only allow play/pause toggle if the timer hasn't finished yet
            if total_elapsed < Duration::from_secs(state.target_seconds as u64) {
                if let Some(start) = state.start_time.take() {
                    state.elapsed += start.elapsed();
                } else {
                    state.start_time = Some(Instant::now());
                }
            }
        }

        // Go back to edit mode only when paused or finished
        if events.contains(UiEvents::UP) && state.start_time.is_none() {
            state.is_editing = true;
        }
    }

    if events.contains(UiEvents::KEY_ESC) {
        return Some(App::time_tools_menu());
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &CountdownState) {
    // draw current time
    let sub_display_bounds = ctx.display_0_96.rect();
    let mut sub_ui = Ui::new(&mut ctx.display_0_96, ctx.font_large);

    let current_time = Local::now();
    let current_time_str = current_time.format("%H:%M:%S").to_string();

    sub_ui
        .label(sub_display_bounds, &current_time_str)
        .center()
        .draw();

    // draw main display
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut header_rect = Rect::default();
    let mut divider_rect = Rect::default();
    let mut content_rect = Rect::default();
    let mut bottom_divider_rect = Rect::default();
    let mut footer_rect = Rect::default();

    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, HEADER_HEIGHT)
                .assign_to(&mut header_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, TOP_DIVIDER_HEIGHT)
                .assign_to(&mut divider_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut content_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, BOTTOM_DIVIDER_HEIGHT)
                .assign_to(&mut bottom_divider_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, FOOTER_HEIGHT)
                .assign_to(&mut footer_rect),
        )
        .layout(display_bounds);

    ui.label(header_rect, "COUNTDOWN").center().draw();
    ui.divider(divider_rect);

    // Calculate elapsed and remaining durations
    let total_elapsed = match state.start_time {
        Some(start) => state.elapsed + start.elapsed(),
        None => state.elapsed,
    };

    let target_duration = Duration::from_secs(state.target_seconds as u64);
    let is_finished = total_elapsed >= target_duration;

    let display_seconds = if state.is_editing {
        state.target_seconds
    } else {
        let remaining = target_duration.saturating_sub(total_elapsed);
        let mut secs = remaining.as_secs();
        // Replicate `div_ceil` behavior: round up if there are fractional sub-seconds left
        if remaining.subsec_nanos() > 0 {
            secs += 1;
        }
        secs as u32
    };

    let mins = display_seconds / 60;
    let secs = display_seconds % 60;

    let time_str = if state.is_editing {
        if state.edit_field == 0 {
            format!(">[{:02}]:{:02}", mins, secs)
        } else {
            format!(" [{:02}]:>{:02}", mins, secs)
        }
    } else if is_finished {
        "TIME UP!".to_string()
    } else {
        format!("{:02}:{:02}", mins, secs)
    };

    ui.label(content_rect, &time_str).center().draw();
    ui.divider(bottom_divider_rect);

    // Footer Hint
    let is_running = state.start_time.is_some();
    let footer_text = if state.is_editing {
        "[Up/Dn] Set  [OK] Start"
    } else if is_finished {
        "[Up] Set New Timer"
    } else if is_running {
        "[OK] Pause  [Esc] Back"
    } else {
        "[OK] Resume  [Up] Edit"
    };
    ui.label(footer_rect, footer_text)
        .scroll(state.tick, 1)
        .draw();
}
