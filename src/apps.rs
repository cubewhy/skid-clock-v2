use crate::{
    app_context::{AppContext, UpdateContext},
    apps::settings::SettingsState,
};

mod clock;
mod main_menu;
mod settings;

pub enum App {
    Clock,
    MainMenu { selected_index: i32 },
    TimeToolsMenu,
    GamesMenu,
    Settings(SettingsState),
}

impl App {
    pub fn update(&mut self, ctx: &mut UpdateContext) -> Option<App> {
        let events = ctx.events;
        match self {
            App::MainMenu { selected_index } => main_menu::update(events, selected_index),
            App::Clock => clock::update(events),
            App::TimeToolsMenu => Some(App::MainMenu { selected_index: 0 }),
            App::GamesMenu => Some(App::MainMenu { selected_index: 0 }),
            App::Settings(state) => settings::update(events, ctx.rtc, state),
        }
    }

    pub fn draw(&self, ctx: &mut AppContext) -> anyhow::Result<()> {
        match self {
            App::MainMenu { selected_index } => main_menu::draw(ctx, *selected_index),
            App::Clock => clock::draw(ctx),
            App::TimeToolsMenu => {}
            App::GamesMenu => {}
            App::Settings(state) => settings::draw(ctx, state),
        }
        Ok(())
    }
}
