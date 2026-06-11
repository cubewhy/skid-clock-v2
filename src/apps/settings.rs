use crate::{
    app_context::AppContext,
    apps::App,
    rtc::ds1302::Ds1302,
    ui::{Ui, UiEvents},
};

#[derive(Default)]
pub struct SettingsState {
    // TODO: hms?
}

pub fn update(events: UiEvents, rtc: &mut Ds1302, state: &mut SettingsState) -> Option<App> {
    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &SettingsState) {
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);
}
