use crate::{
    app_context::AppContext,
    apps::App,
    ui::{Ui, ctx::UiEvents},
};

pub fn update(event: UiEvents, selected_index: &mut i32) -> Option<App> {
    if event.contains(UiEvents::PRIMARY_UP) && *selected_index > 0 {
        *selected_index -= 1;
    }

    if event.contains(UiEvents::PRIMARY_DOWN) && *selected_index < 2 {
        *selected_index += 1;
    }

    if event.contains(UiEvents::PRIMARY_CONFIRM) {
        return (*selected_index == 0).then_some(App::Clock);
    }

    None
}

pub fn draw(ctx: &mut AppContext, selected_index: i32) {
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    // let rect = Rect::new(0, 0, ?, ?);

    // TODO: add flexbox in layout.rs

    // TODO: draw line/rect api in Ui
    // ui.label(rect, "SYSTEM MENU");
}
