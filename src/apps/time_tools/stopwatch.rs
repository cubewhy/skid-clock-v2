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

pub struct LapRecord {
    pub id: usize,
    pub duration: Duration,
    pub cumulative: Duration,
}

pub struct StopwatchState {
    pub tick: u32,
    pub elapsed: Duration,
    pub start_time: Option<Instant>,
    pub laps: Vec<LapRecord>,
    pub selected_lap_index: usize,
}

impl Default for StopwatchState {
    fn default() -> Self {
        Self {
            tick: 0,
            elapsed: Duration::ZERO,
            start_time: None,
            laps: Vec::new(),
            selected_lap_index: 0,
        }
    }
}

pub fn update(ctx: &UpdateContext, state: &mut StopwatchState) -> Option<App> {
    state.tick += 1;
    let events = ctx.menu_events;

    // ESC to return to time tools menu
    if events.intersects(UiEvents::KEY_ESC) {
        return Some(App::time_tools_menu());
    }

    // CONFIRM or KEY_7 to toggle Play/Pause
    if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) {
        if let Some(start) = state.start_time.take() {
            // Pause: Accumulate the time passed since the last start
            state.elapsed += start.elapsed();
        } else {
            // Start/Resume: Record the current instant
            state.start_time = Some(Instant::now());
        }
    }

    // KEY_4 to record a Lap Split
    if events.contains(UiEvents::KEY_4) {
        let current_total = match state.start_time {
            Some(start) => state.elapsed + start.elapsed(),
            None => state.elapsed,
        };

        if current_total > Duration::ZERO {
            let last_cumulative = state
                .laps
                .last()
                .map(|l| l.cumulative)
                .unwrap_or(Duration::ZERO);
            let lap_duration = current_total.saturating_sub(last_cumulative);

            state.laps.push(LapRecord {
                id: state.laps.len() + 1,
                duration: lap_duration,
                cumulative: current_total,
            });

            // Automatically scroll selection down to the newest lap record
            state.selected_lap_index = state.laps.len().saturating_sub(1);
        }
    }

    // Lap List Scrolling: KEY_1 (Up) and KEY_5 (Down)
    if !state.laps.is_empty() {
        if events.contains(UiEvents::KEY_1) {
            state.selected_lap_index = state.selected_lap_index.saturating_sub(1);
        }
        if events.contains(UiEvents::KEY_5) {
            state.selected_lap_index = (state.selected_lap_index + 1).min(state.laps.len() - 1);
        }
    }

    // UP button to clear / reset stopwatch when paused
    if events.contains(UiEvents::UP) && state.start_time.is_none() {
        state.elapsed = Duration::ZERO;
        state.laps.clear();
        state.selected_lap_index = 0;
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &StopwatchState) {
    let sub_display_bounds = ctx.display_0_96.rect();

    // Divide sub-display bounds manually into Clock header and Scroll List body
    let clock_container_height = 16;
    let clock_rect = Rect::new(
        sub_display_bounds.x,
        sub_display_bounds.y,
        sub_display_bounds.width,
        clock_container_height,
    );
    let list_rect = Rect::new(
        sub_display_bounds.x,
        sub_display_bounds.y + clock_container_height as i32,
        sub_display_bounds.width,
        sub_display_bounds
            .height
            .saturating_sub(clock_container_height),
    );

    // Draw real-world wall clock at the top using font_large
    let mut sub_clock_ui = Ui::new(&mut ctx.display_0_96, ctx.font_large);
    let current_time = Local::now();
    let current_time_str = current_time.format("%H:%M:%S").to_string();
    sub_clock_ui
        .label(clock_rect, &current_time_str)
        .center()
        .draw();

    // Draw Lap scrollable list below the clock using standard font
    let mut sub_list_ui = Ui::new(&mut ctx.display_0_96, ctx.font);
    let item_height = 12;
    let visible_count = (list_rect.height / item_height) as usize;
    let visible_count = visible_count.max(1);

    sub_list_ui.scroll_list(
        list_rect,
        &state.laps,
        state.selected_lap_index,
        visible_count,
        item_height,
        |ui, item_rect, item, is_selected| {
            let total_seconds = item.duration.as_secs();
            let hundredths = item.duration.subsec_millis() / 10;
            let seconds = total_seconds % 60;
            let minutes = (total_seconds / 60) % 60;

            let prefix = if is_selected { ">" } else { " " };
            let lap_str = format!(
                "{}L{:02} {:02}:{:02}.{:02}",
                prefix, item.id, minutes, seconds, hundredths
            );
            ui.label(item_rect, &lap_str).draw();
        },
    );

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

    // Header
    ui.label(header_rect, "STOPWATCH").center().draw();
    ui.horizontal_divider(divider_rect);

    // Calculate total elapsed time dynamically
    let total_elapsed = match state.start_time {
        Some(start) => state.elapsed + start.elapsed(),
        None => state.elapsed,
    };

    let total_seconds = total_elapsed.as_secs();
    let hundredths = total_elapsed.subsec_millis() / 10;
    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3600;

    let time_str = if hours > 0 {
        format!(
            "{:02}:{:02}:{:02}.{:02}",
            hours, minutes, seconds, hundredths
        )
    } else {
        format!("{:02}:{:02}.{:02}", minutes, seconds, hundredths)
    };

    // Render large centered stopwatch display
    ui.label(content_rect, &time_str).center().draw();
    ui.horizontal_divider(bottom_divider_rect);

    // Footer Hint
    let is_running = state.start_time.is_some();
    let footer_text = if is_running {
        "[7] Pause [4] Lap [1/5] Scroll"
    } else if total_elapsed > Duration::ZERO {
        "[7] Resume [Up] Reset [1/5] Scroll"
    } else {
        "[7] Start  [Esc] Back"
    };
    ui.label(footer_rect, footer_text)
        .scroll(state.tick, 1)
        .draw();
}
