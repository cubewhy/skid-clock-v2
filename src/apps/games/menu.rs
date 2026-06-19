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
pub enum GamesMenuItem {
    Snake,
    Tetris,
    Pong,
    Dino,
    Stack,
    StickNeedle,
    Target,
}

impl GamesMenuItem {
    const ALL: &[Self] = &[
        Self::Snake,
        Self::Tetris,
        Self::Pong,
        Self::Dino,
        Self::Stack,
        Self::StickNeedle,
        Self::Target,
    ];

    const fn title(&self) -> &'static str {
        match self {
            Self::Snake => "Snake Game",
            Self::Tetris => "Tetris Block",
            Self::Pong => "Retro Pong",
            Self::Dino => "chrome://dino",
            Self::Stack => "Stack Tower",
            Self::StickNeedle => "Stick Needle",
            Self::Target => "Target",
        }
    }

    fn to_app(self) -> App {
        match self {
            Self::Snake => App::snake_game(),
            Self::Tetris => App::tetris_game(),
            Self::Pong => App::pong_game(),
            Self::Dino => App::dino_game(),
            Self::Stack => App::stack_game(),
            Self::StickNeedle => App::stick_needle_game(),
            Self::Target => App::target_game(),
        }
    }
}

const HEADER_HEIGHT: u32 = 15;
const TOP_DIVIDER_HEIGHT: u32 = 2;

const ITEM_HEIGHT: u32 = 10;
const VISIBLE_COUNT: usize = 4;

#[derive(Default)]
pub struct GamesMenuState {
    pub select_index: u8,
    pub tick: u32,
}

pub fn update(ctx: &UpdateContext, state: &mut GamesMenuState) -> Option<App> {
    state.tick += 1;

    let select_index = &mut state.select_index;
    let events = ctx.menu_events;

    let max_index = (GamesMenuItem::ALL.len() - 1) as u8;

    if events.contains(UiEvents::UP) {
        if *select_index > 0 {
            *select_index -= 1;
        } else {
            *select_index = max_index;
        }
    }

    if events.contains(UiEvents::DOWN) {
        if *select_index < max_index {
            *select_index += 1;
        } else {
            *select_index = 0;
        }
    }

    // Pressing ESC or LEFT returns you to the Main Menu
    if events.intersects(UiEvents::KEY_ESC | UiEvents::LEFT) {
        return Some(App::main_menu());
    }

    if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7 | UiEvents::RIGHT)
        && let Some(item) = GamesMenuItem::ALL.get(*select_index as usize)
    {
        return Some(item.to_app());
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &GamesMenuState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut header_rect = Rect::default();
    let mut divider_rect = Rect::default();
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
        .layout(display_bounds);

    ui.label(header_rect, "ARCADE GAMES").center().draw();
    ui.divider(divider_rect);

    let mut menu_titles = [""; GamesMenuItem::ALL.len()];
    let mut i = 0;
    while i < GamesMenuItem::ALL.len() {
        menu_titles[i] = GamesMenuItem::ALL[i].title();
        i += 1;
    }

    ui.scroll_list(
        list_rect,
        &menu_titles,
        state.select_index as usize,
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
