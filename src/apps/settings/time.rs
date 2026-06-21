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

pub struct TimeSettingsState {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub selected_field: usize,
    pub last_update: Option<Instant>,
    pub is_initialized: bool,
}

impl TimeSettingsState {
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

impl Default for TimeSettingsState {
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

fn clamp_day_for_month(state: &mut TimeSettingsState) {
    let max_days = days_in_month(state.year as i32, state.month as u32) as u8;
    if state.day > max_days {
        state.day = max_days;
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut TimeSettingsState) -> Option<App> {
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

    if let Some(last_anchor) = state.last_update {
        let elapsed_seconds = last_anchor.elapsed().as_secs();
        if elapsed_seconds > 0 {
            let current_dt = state.get_datetime();
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
        return Some(App::settings_menu());
    }

    if ctx
        .menu_events
        .intersects(UiEvents::KEY_7 | UiEvents::CONFIRM)
    {
        let _ = ctx.rtc.set_time(&state.get_datetime());
        let _ = sync_time(ctx.rtc);
        return Some(App::settings_menu());
    }

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

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &TimeSettingsState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

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

    let mut y_bytes = [b'0'; 4];
    let mut temp_y = state.year as u32;
    for i in (0..4).rev() {
        y_bytes[i] = b'0' + (temp_y % 10) as u8;
        temp_y /= 10;
    }
    let y_str = core::str::from_utf8(&y_bytes).unwrap_or("2026");

    let mo_str = format!("{:02}", state.month);
    let d_str = format!("{:02}", state.day);
    let hh_str = format!("{:02}", state.hours);
    let mm_str = format!("{:02}", state.minutes);
    let ss_str = format!("{:02}", state.seconds);

    ui.label(rect_title, "DATE & TIME SETTINGS").center().draw();
    ui.divider(rect_divider);

    ui.button(rect_year, y_str, state.selected_field == 0);
    ui.label(rect_d1, "-").center().draw();
    ui.button(rect_month, &mo_str, state.selected_field == 1);
    ui.label(rect_d2, "-").center().draw();
    ui.button(rect_day, &d_str, state.selected_field == 2);

    ui.button(rect_hh, &hh_str, state.selected_field == 3);
    ui.label(rect_c1, ":").center().draw();
    ui.button(rect_mm, &mm_str, state.selected_field == 4);
    ui.label(rect_c2, ":").center().draw();
    ui.button(rect_ss, &ss_str, state.selected_field == 5);
}
