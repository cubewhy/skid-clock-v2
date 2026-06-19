use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    ui::Ui,
};

#[derive(Default)]
pub struct SettingsState {
    // TODO: hms?
}

pub fn update(ctx: &UpdateContext, state: &mut SettingsState) -> Option<App> {
    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &SettingsState) {
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);
}
