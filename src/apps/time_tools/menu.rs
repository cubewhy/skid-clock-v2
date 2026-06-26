use std::sync::atomic::{AtomicU8, Ordering};

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

static SAVED_MENU_INDEX: AtomicU8 = AtomicU8::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimeToolsItem {
    Stopwatch,
    Countdown,
    Pomodoro,
}

impl TimeToolsItem {
    const ALL: &[Self] = &[Self::Stopwatch, Self::Countdown, Self::Pomodoro];

    const fn title(&self) -> &'static str {
        match self {
            Self::Stopwatch => "Stopwatch",
            Self::Countdown => "Countdown",
            Self::Pomodoro => "Pomodoro",
        }
    }

    fn to_app(self) -> App {
        match self {
            Self::Stopwatch => App::stopwatch(),
            Self::Countdown => App::countdown(),
            Self::Pomodoro => App::pomodoro(),
        }
    }
}

const HEADER_HEIGHT: u32 = 15;
const TOP_DIVIDER_HEIGHT: u32 = 2;
const BOTTOM_DIVIDER_HEIGHT: u32 = 1;

const ITEM_HEIGHT: u32 = 10;
const VISIBLE_COUNT: usize = 4;

pub struct TimeToolsMenuState {
    pub selected_index: u8,
    pub tick: u32,
}

impl Default for TimeToolsMenuState {
    fn default() -> Self {
        Self {
            selected_index: SAVED_MENU_INDEX.load(Ordering::Relaxed),
            tick: 0,
        }
    }
}

pub fn update(ctx: &UpdateContext, state: &mut TimeToolsMenuState) -> Option<App> {
    state.tick += 1;

    let selected_index = &mut state.selected_index;
    let events = ctx.menu_events;

    let max_index = (TimeToolsItem::ALL.len() - 1) as u8;
    let mut index_changed = false;

    if events.intersects(UiEvents::UP | UiEvents::KEY_6) {
        if *selected_index > 0 {
            *selected_index -= 1;
        } else {
            *selected_index = max_index;
        }
        index_changed = true;
    }

    if events.intersects(UiEvents::DOWN | UiEvents::KEY_5) {
        if *selected_index < max_index {
            *selected_index += 1;
        } else {
            *selected_index = 0;
        }
        index_changed = true;
    }

    if index_changed {
        SAVED_MENU_INDEX.store(*selected_index, Ordering::Relaxed);
    }

    if events.intersects(UiEvents::KEY_ESC | UiEvents::LEFT | UiEvents::KEY_4) {
        return Some(App::main_menu());
    }

    if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7 | UiEvents::RIGHT)
        && let Some(item) = TimeToolsItem::ALL.get(*selected_index as usize)
    {
        return Some(item.to_app());
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &TimeToolsMenuState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut header_rect = Rect::default();
    let mut divider_rect = Rect::default();
    let mut bottom_divider_rect = Rect::default();
    let mut list_rect = Rect::default();

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
                .assign_to(&mut list_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, BOTTOM_DIVIDER_HEIGHT)
                .assign_to(&mut bottom_divider_rect),
        )
        .layout(display_bounds);

    ui.label(header_rect, "TIME TOOLS").center().draw();
    ui.horizontal_divider(divider_rect);

    let mut menu_titles = [""; TimeToolsItem::ALL.len()];
    let mut i = 0;
    while i < TimeToolsItem::ALL.len() {
        menu_titles[i] = TimeToolsItem::ALL[i].title();
        i += 1;
    }

    ui.scroll_list(
        list_rect,
        &menu_titles,
        state.selected_index as usize,
        VISIBLE_COUNT,
        ITEM_HEIGHT,
        |ui_ctx, item_rect, item_name, is_selected| {
            if is_selected {
                ui_ctx.label(item_rect, &format!("> {}", item_name)).draw();
            } else {
                ui_ctx.label(item_rect, &format!("  {}", item_name)).draw();
            }
        },
    );
}
