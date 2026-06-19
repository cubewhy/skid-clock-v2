use crate::display::UnifiedDisplay;
use crate::rtc::{UnifiedRtc, sync_time};
use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    ui::{
        Rect, Ui, UiEvents,
        layout::{AlignItems, FlexDirection, FlexNode},
    },
};
use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use std::time::Instant;

pub struct SettingsState {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub selected_field: usize, // 0=Year, 1=Month, 2=Day, 3=Hour, 4=Minute, 5=Second
    pub last_update: Option<Instant>, // Monotonic time reference anchor
    pub is_initialized: bool,
}

impl SettingsState {
    /// Combines the interactive state parameters into a unified NaiveDateTime
    pub fn get_datetime(&self) -> chrono::NaiveDateTime {
        let configured_date =
            NaiveDate::from_ymd_opt(self.year as i32, self.month as u32, self.day as u32)
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());

        let configured_time =
            NaiveTime::from_hms_opt(self.hours as u32, self.minutes as u32, self.seconds as u32)
                .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());

        NaiveDateTime::new(configured_date, configured_time)
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            year: 2026,
            month: 1,
            day: 1,
            hours: 12,
            minutes: 0,
            seconds: 0,
            selected_field: 0,
            last_update: None,
            is_initialized: false,
        }
    }
}

/// Helper function to retrieve max days within a specific calendar month
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 31,
    }
}

/// Clamps the active day value safely if year/month mutations invalidate it
fn clamp_day_for_month(state: &mut SettingsState) {
    let max_days = days_in_month(state.year as i32, state.month as u32) as u8;
    if state.day > max_days {
        state.day = max_days;
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut SettingsState) -> Option<App> {
    if !state.is_initialized {
        let current_system_time = Local::now();
        state.year = current_system_time.year() as u16;
        state.month = current_system_time.month() as u8;
        state.day = current_system_time.day() as u8;
        state.hours = current_system_time.hour() as u8;
        state.minutes = current_system_time.minute() as u8;
        state.seconds = current_system_time.second() as u8;
        state.last_update = Some(Instant::now());
        state.is_initialized = true;
    }

    // Continuous time propagation handling cross-boundary transitions dynamically
    if let Some(last_anchor) = state.last_update {
        let elapsed_seconds = last_anchor.elapsed().as_secs();
        if elapsed_seconds > 0 {
            let current_dt = state.get_datetime();

            // Advance system time using safe signed duration shifts
            if let Some(updated_dt) =
                current_dt.checked_add_signed(chrono::Duration::seconds(elapsed_seconds as i64))
            {
                state.year = updated_dt.year() as u16;
                state.month = updated_dt.month() as u8;
                state.day = updated_dt.day() as u8;
                state.hours = updated_dt.hour() as u8;
                state.minutes = updated_dt.minute() as u8;
                state.seconds = updated_dt.second() as u8;
            }
            state.last_update = Some(Instant::now());
        }
    }

    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::main_menu());
    }

    if ctx
        .menu_events
        .intersects(UiEvents::KEY_7 | UiEvents::CONFIRM)
    {
        match ctx.rtc.set_time(&state.get_datetime()) {
            Ok(_) => {}
            Err(err) => log::error!("Failed to sync time to DS1302: {err:#}"),
        };
        match sync_time(ctx.rtc) {
            Ok(_) => {}
            Err(err) => log::error!("Failed to sync time to FreeRTOS: {err:#}"),
        };
        return Some(App::main_menu());
    }

    // Cursor navigation selection loop (0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 0)
    if ctx.menu_events.contains(UiEvents::LEFT) {
        state.selected_field = if state.selected_field == 0 {
            5
        } else {
            state.selected_field - 1
        };
    }
    if ctx.menu_events.contains(UiEvents::RIGHT) {
        state.selected_field = (state.selected_field + 1) % 6;
    }

    // Value modification mechanics
    if ctx.menu_events.contains(UiEvents::UP) {
        match state.selected_field {
            0 => {
                state.year = if state.year >= 2099 {
                    2000
                } else {
                    state.year + 1
                };
                clamp_day_for_month(state);
            }
            1 => {
                state.month = if state.month >= 12 {
                    1
                } else {
                    state.month + 1
                };
                clamp_day_for_month(state);
            }
            2 => {
                let max_d = days_in_month(state.year as i32, state.month as u32);
                state.day = if state.day as u32 >= max_d {
                    1
                } else {
                    state.day + 1
                };
            }
            3 => state.hours = (state.hours + 1) % 24,
            4 => state.minutes = (state.minutes + 1) % 60,
            5 => state.seconds = (state.seconds + 1) % 60,
            _ => {}
        }
    }

    if ctx.menu_events.contains(UiEvents::DOWN) {
        match state.selected_field {
            0 => {
                state.year = if state.year <= 2000 {
                    2099
                } else {
                    state.year - 1
                };
                clamp_day_for_month(state);
            }
            1 => {
                state.month = if state.month <= 1 {
                    12
                } else {
                    state.month - 1
                };
                clamp_day_for_month(state);
            }
            2 => {
                let max_d = days_in_month(state.year as i32, state.month as u32);
                state.day = if state.day <= 1 {
                    max_d as u8
                } else {
                    state.day - 1
                };
            }
            3 => {
                state.hours = if state.hours == 0 {
                    23
                } else {
                    state.hours - 1
                }
            }
            4 => {
                state.minutes = if state.minutes == 0 {
                    59
                } else {
                    state.minutes - 1
                }
            }
            5 => {
                state.seconds = if state.seconds == 0 {
                    59
                } else {
                    state.seconds - 1
                }
            }
            _ => {}
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &SettingsState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    // Stack allocation variables for Flexbox engine mapping targets
    let mut rect_title = Rect::default();
    let mut rect_divider = Rect::default();
    let mut rect_footer = Rect::default();

    let mut rect_year = Rect::default();
    let mut rect_d1 = Rect::default();
    let mut rect_month = Rect::default();
    let mut rect_d2 = Rect::default();
    let mut rect_day = Rect::default();

    let mut rect_hh = Rect::default();
    let mut rect_c1 = Rect::default();
    let mut rect_mm = Rect::default();
    let mut rect_c2 = Rect::default();
    let mut rect_ss = Rect::default();

    // Multi-row architectural hierarchy tree layout
    let root = FlexNode::new(FlexDirection::Column)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 11)
                .assign_to(&mut rect_title),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 2)
                .assign_to(&mut rect_divider),
        )
        // Date Input Selector Container
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .align_items(AlignItems::Center)
                .child(FlexNode::new(FlexDirection::Row).with_flex(1))
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(34, 14)
                        .assign_to(&mut rect_year),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(8, 14)
                        .assign_to(&mut rect_d1),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(20, 14)
                        .assign_to(&mut rect_month),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(8, 14)
                        .assign_to(&mut rect_d2),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(20, 14)
                        .assign_to(&mut rect_day),
                )
                .child(FlexNode::new(FlexDirection::Row).with_flex(1)),
        )
        // Time Input Selector Container
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .align_items(AlignItems::Center)
                .child(FlexNode::new(FlexDirection::Row).with_flex(1))
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(20, 14)
                        .assign_to(&mut rect_hh),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(8, 14)
                        .assign_to(&mut rect_c1),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(20, 14)
                        .assign_to(&mut rect_mm),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(8, 14)
                        .assign_to(&mut rect_c2),
                )
                .child(
                    FlexNode::new(FlexDirection::Row)
                        .with_size(20, 14)
                        .assign_to(&mut rect_ss),
                )
                .child(FlexNode::new(FlexDirection::Row).with_flex(1)),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 11)
                .assign_to(&mut rect_footer),
        );

    root.layout(display_bounds);

    // TODO: replace with a crate
    let mut y_bytes = [b'0'; 4];
    let mut temp_y = state.year as u32;
    y_bytes[3] = b'0' + (temp_y % 10) as u8;
    temp_y /= 10;
    y_bytes[2] = b'0' + (temp_y % 10) as u8;
    temp_y /= 10;
    y_bytes[1] = b'0' + (temp_y % 10) as u8;
    temp_y /= 10;
    y_bytes[0] = b'0' + (temp_y % 10) as u8;
    let y_str = core::str::from_utf8(&y_bytes).unwrap_or("2026");

    let mut mo_bytes = [b'0', b'0'];
    mo_bytes[0] += state.month / 10;
    mo_bytes[1] += state.month % 10;
    let mo_str = core::str::from_utf8(&mo_bytes).unwrap_or("01");

    let mut d_bytes = [b'0', b'0'];
    d_bytes[0] += state.day / 10;
    d_bytes[1] += state.day % 10;
    let d_str = core::str::from_utf8(&d_bytes).unwrap_or("01");

    let mut hh_bytes = [b'0', b'0'];
    hh_bytes[0] += state.hours / 10;
    hh_bytes[1] += state.hours % 10;
    let hh_str = core::str::from_utf8(&hh_bytes).unwrap_or("00");

    let mut mm_bytes = [b'0', b'0'];
    mm_bytes[0] += state.minutes / 10;
    mm_bytes[1] += state.minutes % 10;
    let mm_str = core::str::from_utf8(&mm_bytes).unwrap_or("00");

    let mut ss_bytes = [b'0', b'0'];
    ss_bytes[0] += state.seconds / 10;
    ss_bytes[1] += state.seconds % 10;
    let ss_str = core::str::from_utf8(&ss_bytes).unwrap_or("00");

    // Pass 1: Render structural headers
    ui.label(rect_title, "DATE & TIME SETTINGS").center().draw();
    ui.divider(rect_divider);

    // Pass 2: Render date selector sequence components
    ui.button(rect_year, y_str, state.selected_field == 0);
    ui.label(rect_d1, "-").center().draw();
    ui.button(rect_month, mo_str, state.selected_field == 1);
    ui.label(rect_d2, "-").center().draw();
    ui.button(rect_day, d_str, state.selected_field == 2);

    // Pass 3: Render time selector sequence components
    ui.button(rect_hh, hh_str, state.selected_field == 3);
    ui.label(rect_c1, ":").center().draw();
    ui.button(rect_mm, mm_str, state.selected_field == 4);
    ui.label(rect_c2, ":").center().draw();
    ui.button(rect_ss, ss_str, state.selected_field == 5);

    // Pass 4: Render operating system manual footer shortcuts
    ui.label(rect_footer, "[-]Adj [*]Move [7]Save [Esc]")
        .center()
        .draw();
}
