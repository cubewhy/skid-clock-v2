use crate::{
    app_context::{AppContext, UpdateContext},
    apps::{main_menu::MainMenuState, settings::SettingsState},
};

mod clock;
mod main_menu;
mod settings;

pub enum App {
    Clock,
    MainMenu(MainMenuState),
    TimeToolsMenu,
    GamesMenu,
    Settings(SettingsState),
}

impl App {
    pub fn update(&mut self, ctx: &mut UpdateContext) -> Option<App> {
        match self {
            App::MainMenu(state) => main_menu::update(ctx, state),
            App::Clock => clock::update(ctx),
            App::TimeToolsMenu => Some(App::main_menu()),
            App::GamesMenu => Some(App::main_menu()),
            App::Settings(state) => settings::update(ctx, state),
        }
    }

    pub fn draw(&self, ctx: &mut AppContext) -> anyhow::Result<()> {
        match self {
            App::MainMenu(state) => main_menu::draw(ctx, state),
            App::Clock => clock::draw(ctx),
            App::TimeToolsMenu => {}
            App::GamesMenu => {}
            App::Settings(state) => settings::draw(ctx, state),
        }
        Ok(())
    }

    pub fn main_menu() -> Self {
        Self::MainMenu(MainMenuState::default())
    }
}
