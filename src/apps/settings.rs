use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    ui::{Ui, UiEvents},
};

#[derive(Default)]
pub struct SettingsState {
    // TODO: hms?
}

pub fn update(ctx: &UpdateContext, state: &mut SettingsState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::main_menu());
    }
    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &SettingsState) {
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);
}
