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

const WORK_DURATION_SECS: u64 = 25 * 60;
const BREAK_DURATION_SECS: u64 = 5 * 60;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PomodoroStage {
    Work,
    Break,
}

pub struct PomodoroState {
    pub tick: u32,
    pub stage: PomodoroStage,
    pub elapsed: Duration,
    pub start_time: Option<Instant>,
    pub completed_sessions: u32,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self {
            tick: 0,
            stage: PomodoroStage::Work,
            elapsed: Duration::ZERO,
            start_time: None,
            completed_sessions: 0,
        }
    }
}

pub fn update(ctx: &UpdateContext, state: &mut PomodoroState) -> Option<App> {
    state.tick += 1;
    let events = ctx.menu_events;

    let target_duration = match state.stage {
        PomodoroStage::Work => Duration::from_secs(WORK_DURATION_SECS),
        PomodoroStage::Break => Duration::from_secs(BREAK_DURATION_SECS),
    };

    // Auto-advance stage when target duration is completed
    if let Some(start) = state.start_time {
        let total_elapsed = state.elapsed + start.elapsed();
        if total_elapsed >= target_duration {
            match state.stage {
                PomodoroStage::Work => {
                    state.stage = PomodoroStage::Break;
                    state.completed_sessions += 1;
                }
                PomodoroStage::Break => {
                    state.stage = PomodoroStage::Work;
                }
            }
            state.elapsed = Duration::ZERO;
            state.start_time = None; // Pause upon completing a transition stage
        }
    }

    if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) {
        if let Some(start) = state.start_time.take() {
            state.elapsed += start.elapsed();
        } else {
            state.start_time = Some(Instant::now());
        }
    }

    // UP resets the complete pomodoro state back to defaults when paused
    if events.contains(UiEvents::UP) && state.start_time.is_none() {
        *state = PomodoroState::default();
    }

    // ESC or LEFT exits back to the time tools menu
    if events.intersects(UiEvents::KEY_ESC | UiEvents::LEFT) {
        return Some(App::time_tools_menu());
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &PomodoroState) {
    let sub_display_bounds = ctx.display_0_96.rect();
    let mut sub_ui = Ui::new(&mut ctx.display_0_96, ctx.font_large);

    let current_time = Local::now();
    let current_time_str = current_time.format("%H:%M:%S").to_string();

    sub_ui
        .label(sub_display_bounds, &current_time_str)
        .center()
        .draw();

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

    // Header updates dynamically to state phase
    let header_title = match state.stage {
        PomodoroStage::Work => format!("POMODORO [WORK #{}]", state.completed_sessions + 1),
        PomodoroStage::Break => "POMODORO [BREAK]".to_string(),
    };
    ui.label(header_rect, &header_title).center().draw();
    ui.divider(divider_rect);

    // Calculate total runtime configurations safely
    let target_duration = match state.stage {
        PomodoroStage::Work => Duration::from_secs(WORK_DURATION_SECS),
        PomodoroStage::Break => Duration::from_secs(BREAK_DURATION_SECS),
    };

    let total_elapsed = match state.start_time {
        Some(start) => state.elapsed + start.elapsed(),
        None => state.elapsed,
    };

    let remaining = target_duration.saturating_sub(total_elapsed);
    let mut display_seconds = remaining.as_secs();

    // Maintain precise ceiling rounding matching original behavior
    if remaining.subsec_nanos() > 0 {
        display_seconds += 1;
    }

    let mins = display_seconds / 60;
    let secs = display_seconds % 60;
    let time_str = format!("{:02}:{:02}", mins, secs);

    ui.label(content_rect, &time_str).center().draw();
    ui.divider(bottom_divider_rect);

    // Footer Hint
    let is_running = state.start_time.is_some();
    let footer_text = if is_running {
        "[OK] Pause Session"
    } else if state.elapsed == Duration::ZERO && state.completed_sessions == 0 {
        "[OK] Start Work"
    } else {
        "[OK] Resume  [Up] Reset Session"
    };
    ui.label(footer_rect, footer_text)
        .scroll(state.tick, 1)
        .draw();
}
