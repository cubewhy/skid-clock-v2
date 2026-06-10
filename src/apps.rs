use crate::app_context::{AppContext, UpdateContext};

mod clock;
mod main_menu;

pub enum App {
    Clock,
    MainMenu { selected_index: i32 },
}

impl App {
    pub fn update(&mut self, ctx: &mut UpdateContext) -> Option<App> {
        let event = ctx.event;
        match self {
            App::MainMenu { selected_index } => main_menu::update(event, selected_index),
            App::Clock => clock::update(event),
        }
    }

    pub fn draw(&self, ctx: &mut AppContext) -> anyhow::Result<()> {
        match self {
            App::MainMenu { selected_index } => main_menu::draw(ctx, *selected_index),
            App::Clock => clock::draw(ctx),
        }
        Ok(())
    }
}
