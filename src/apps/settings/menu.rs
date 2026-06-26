use std::sync::atomic::{AtomicU8, Ordering};

use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::UnifiedDisplay,
    ui::{
        Rect, Ui, UiEvents,
        layout::{FlexDirection, FlexNode},
    },
};

static SAVED_MENU_INDEX: AtomicU8 = AtomicU8::new(0);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SettingsItem {
    TimeSettings,
    NetworkSettings,
}

impl SettingsItem {
    const ALL: &[Self] = &[Self::TimeSettings, Self::NetworkSettings];
    const fn title(&self) -> &'static str {
        match self {
            Self::TimeSettings => "Time Settings",
            Self::NetworkSettings => "Network Settings",
        }
    }
}

pub struct SettingsMenuState {
    pub selected_index: u8,
}

impl Default for SettingsMenuState {
    fn default() -> Self {
        Self {
            selected_index: SAVED_MENU_INDEX.load(Ordering::Relaxed),
        }
    }
}

pub fn update(ctx: &UpdateContext, state: &mut SettingsMenuState) -> Option<App> {
    let events = ctx.menu_events;
    let selected_index = &mut state.selected_index;

    let max_index = (SettingsItem::ALL.len() - 1) as u8;
    let mut index_changed = false;

    if events.intersects(UiEvents::KEY_ESC | UiEvents::LEFT | UiEvents::KEY_4) {
        return Some(App::main_menu());
    }
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
    if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7 | UiEvents::RIGHT) {
        return match SettingsItem::ALL[*selected_index as usize] {
            SettingsItem::TimeSettings => Some(App::time_settings()),
            SettingsItem::NetworkSettings => Some(App::network_settings()),
        };
    }

    if index_changed {
        SAVED_MENU_INDEX.store(*selected_index, Ordering::Relaxed);
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &SettingsMenuState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut header_rect = Rect::default();
    let mut divider_rect = Rect::default();
    let mut list_rect = Rect::default();

    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 14)
                .assign_to(&mut header_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 2)
                .assign_to(&mut divider_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut list_rect),
        )
        .layout(display_bounds);

    ui.label(header_rect, "SETTINGS").center().draw();
    ui.horizontal_divider(divider_rect);

    let titles = [SettingsItem::ALL[0].title(), SettingsItem::ALL[1].title()];
    ui.scroll_list(
        list_rect,
        &titles,
        state.selected_index as usize,
        4,
        12,
        |ui_ctx, r, text, selected| {
            if selected {
                ui_ctx.label(r, &format!("> {}", text)).draw();
            } else {
                ui_ctx.label(r, &format!("  {}", text)).draw();
            }
        },
    );
}
