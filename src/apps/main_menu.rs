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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MainMenuItem {
    Clock,
    TimeTools,
    ArcadeGames,
    Settings,
}

impl MainMenuItem {
    const ALL: &[Self] = &[
        Self::Clock,
        Self::TimeTools,
        Self::ArcadeGames,
        Self::Settings,
    ];

    const fn title(&self) -> &'static str {
        match self {
            Self::Clock => "Realtime Clock",
            Self::TimeTools => "Time Tools",
            Self::ArcadeGames => "Arcade Games",
            Self::Settings => "Settings",
        }
    }

    fn to_app(self) -> App {
        match self {
            Self::Clock => App::Clock,
            Self::TimeTools => App::time_tools_menu(),
            Self::ArcadeGames => App::games_menu(),
            Self::Settings => App::time_settings(),
        }
    }
}

const HEADER_HEIGHT: u32 = 15;
const TOP_DIVIDER_HEIGHT: u32 = 2;
const BOTTOM_DIVIDER_HEIGHT: u32 = 1;
const FOOTER_HEIGHT: u32 = 14;

const ITEM_HEIGHT: u32 = 10;
const VISIBLE_COUNT: usize = 3;

#[derive(Default)]
pub struct MainMenuState {
    pub selected_index: u8,
    pub tick: u32,
}

pub fn update(ctx: &UpdateContext, state: &mut MainMenuState) -> Option<App> {
    state.tick += 1;

    let selected_index = &mut state.selected_index;
    let events = ctx.menu_events;

    let max_index = (MainMenuItem::ALL.len() - 1) as u8;

    if events.contains(UiEvents::UP) {
        if *selected_index > 0 {
            *selected_index -= 1;
        } else {
            *selected_index = max_index;
        }
    }

    if events.contains(UiEvents::DOWN) {
        if *selected_index < max_index {
            *selected_index += 1;
        } else {
            *selected_index = 0;
        }
    }

    if events.intersects(UiEvents::KEY_ESC | UiEvents::LEFT) {
        return Some(App::Clock);
    }

    if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7 | UiEvents::RIGHT)
        && let Some(item) = MainMenuItem::ALL.get(*selected_index as usize)
    {
        return Some(item.to_app());
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &MainMenuState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut header_rect = Rect::default();
    let mut divider_rect = Rect::default();
    let mut bottom_divider_rect = Rect::default();
    let mut list_rect = Rect::default();
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
                .assign_to(&mut list_rect),
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

    ui.label(header_rect, "SYSTEM MENU").center().draw();
    ui.divider(divider_rect);

    let mut menu_titles = [""; MainMenuItem::ALL.len()];
    let mut i = 0;
    while i < MainMenuItem::ALL.len() {
        menu_titles[i] = MainMenuItem::ALL[i].title();
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

    ui.divider(bottom_divider_rect);

    ui.label(
        footer_rect,
        "gh@cubewhy/skid-clock-v2 - LICENSED UNDER GPL-3.0 - Open Source Hardware \\ Nya~",
    )
    .scroll(state.tick, 5)
    .draw();
}
