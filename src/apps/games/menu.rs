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
pub enum GamesMenuItem {
    Snake,
    Tetris,
    Pong,
    Dino,
    Stack,
    StickNeedle,
    Target,
    JumpJump,
    WhacMole,
    NavalBattle,
    TankTrouble,
    Game2048,
    FlappyBird,
    GoldMiner,
    Gomuku,
    Brick,
    Undertale,
    Pacman,
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
        Self::JumpJump,
        Self::WhacMole,
        Self::NavalBattle,
        Self::TankTrouble,
        Self::Game2048,
        Self::FlappyBird,
        Self::GoldMiner,
        Self::Gomuku,
        Self::Brick,
        Self::Undertale,
        Self::Pacman,
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
            Self::JumpJump => "Jump Jump",
            Self::WhacMole => "Whac A Mole",
            Self::NavalBattle => "Naval Battle",
            Self::TankTrouble => "Tank Trouble",
            Self::Game2048 => "2048",
            Self::FlappyBird => "Flappy Bird",
            Self::GoldMiner => "Gold Miner",
            Self::Gomuku => "Gomuku",
            Self::Brick => "Brick Breaker",
            Self::Undertale => "Undertale",
            Self::Pacman => "Pacman",
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
            Self::JumpJump => App::jump_jump_game(),
            Self::WhacMole => App::whac_mole_game(),
            Self::NavalBattle => App::naval_battle_game(),
            Self::TankTrouble => App::tank_trouble_game(),
            Self::Game2048 => App::game_2048(),
            Self::FlappyBird => App::flappy_bird_game(),
            Self::GoldMiner => App::gold_miner_game(),
            Self::Gomuku => App::gomoku_game(),
            Self::Brick => App::brick_game(),
            Self::Undertale => App::undertale_game(),
            Self::Pacman => App::pacman_game(),
        }
    }
}

const HEADER_HEIGHT: u32 = 15;
const TOP_DIVIDER_HEIGHT: u32 = 2;

const ITEM_HEIGHT: u32 = 10;
const VISIBLE_COUNT: usize = 4;

pub struct GamesMenuState {
    pub selected_index: u8,
    pub tick: u32,
}

impl Default for GamesMenuState {
    fn default() -> Self {
        Self {
            selected_index: SAVED_MENU_INDEX.load(Ordering::Relaxed),
            tick: 0,
        }
    }
}

pub fn update(ctx: &UpdateContext, state: &mut GamesMenuState) -> Option<App> {
    state.tick += 1;
    let selected_index = &mut state.selected_index;
    let events = ctx.menu_events;

    let max_index = (GamesMenuItem::ALL.len() - 1) as u8;
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
        && let Some(item) = GamesMenuItem::ALL.get(*selected_index as usize)
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
    ui.horizontal_divider(divider_rect);

    let mut menu_titles = [""; GamesMenuItem::ALL.len()];
    let mut i = 0;
    while i < GamesMenuItem::ALL.len() {
        menu_titles[i] = GamesMenuItem::ALL[i].title();
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
