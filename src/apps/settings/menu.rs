use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::UnifiedDisplay,
    ui::{
        Rect, Ui, UiEvents,
        layout::{FlexDirection, FlexNode},
    },
};

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

#[derive(Default)]
pub struct SettingsMenuState {
    pub selected_index: u8,
}

pub fn update(ctx: &UpdateContext, state: &mut SettingsMenuState) -> Option<App> {
    if ctx
        .menu_events
        .intersects(UiEvents::KEY_ESC | UiEvents::LEFT)
    {
        return Some(App::main_menu());
    }
    if ctx.menu_events.contains(UiEvents::UP) {
        state.selected_index = if state.selected_index == 0 { 1 } else { 0 };
    }
    if ctx.menu_events.contains(UiEvents::DOWN) {
        state.selected_index = (state.selected_index + 1) % 2;
    }
    if ctx
        .menu_events
        .intersects(UiEvents::CONFIRM | UiEvents::KEY_7 | UiEvents::RIGHT)
    {
        return match SettingsItem::ALL[state.selected_index as usize] {
            SettingsItem::TimeSettings => Some(App::time_settings()),
            SettingsItem::NetworkSettings => Some(App::network_settings()),
        };
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
