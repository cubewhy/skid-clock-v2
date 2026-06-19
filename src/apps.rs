use crate::{
    app_context::{AppContext, UpdateContext},
    apps::{
        main_menu::MainMenuState, settings::TimeSettingsState, time_tools::menu::TimeToolsMenuState,
    },
};

mod clock;
mod main_menu;
mod settings;
mod time_tools;

pub enum App {
    Clock,
    MainMenu(MainMenuState),
    TimeToolsMenu(TimeToolsMenuState),
    Stopwatch,
    Countdown,
    Pomodoro,

    GamesMenu,
    TimeSettings(TimeSettingsState),
}

impl App {
    pub fn update(&mut self, ctx: &mut UpdateContext) -> Option<App> {
        match self {
            App::MainMenu(state) => main_menu::update(ctx, state),
            App::Clock => clock::update(ctx),
            App::TimeToolsMenu(state) => time_tools::menu::update(ctx, state),
            App::Stopwatch => todo!(),
            App::Countdown => todo!(),
            App::Pomodoro => todo!(),
            App::GamesMenu => Some(App::main_menu()),
            App::TimeSettings(state) => settings::update(ctx, state),
        }
    }

    pub fn draw(&self, ctx: &mut AppContext) -> anyhow::Result<()> {
        match self {
            App::MainMenu(state) => main_menu::draw(ctx, state),
            App::Clock => clock::draw(ctx),
            App::TimeToolsMenu(state) => time_tools::menu::draw(ctx, state),
            App::Stopwatch => todo!(),
            App::Countdown => todo!(),
            App::Pomodoro => todo!(),
            App::GamesMenu => {}
            App::TimeSettings(state) => settings::draw(ctx, state),
        }
        Ok(())
    }

    pub fn main_menu() -> Self {
        Self::MainMenu(Default::default())
    }

    pub fn time_tools_menu() -> Self {
        Self::TimeToolsMenu(Default::default())
    }

    pub fn time_settings() -> Self {
        Self::TimeSettings(Default::default())
    }

    pub fn stopwatch() -> Self {
        Self::Stopwatch
    }

    pub fn countdown() -> Self {
        Self::Countdown
    }

    pub fn pomodoro() -> Self {
        Self::Pomodoro
    }
}
