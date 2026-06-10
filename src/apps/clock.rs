use crate::{
    app_context::AppContext,
    apps::App,
    ui::{Rect, Ui, UiEvent},
};

pub fn update(event: UiEvent) -> Option<App> {
    use UiEvent::*;

    if matches!(
        event,
        PrimaryUp | PrimaryConfirm | PrimaryDown | PrimaryLeft | PrimaryRight | Back
    ) {
        Some(App::MainMenu { selected_index: 0 })
    } else {
        None
    }
}

pub fn draw(ctx: &mut AppContext) {
    let now = chrono::Local::now();
    let ymd_text = now.format("%Y-%m-%d").to_string();
    let hms_text = now.format("%H:%M:%S").to_string();

    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let base_rect = Rect::new(0, 5, 120, 18);
    const LINE_HEIGHT: i32 = 20;

    ui.label(base_rect, &ymd_text).center().draw();

    let hms_rect = base_rect.offset(0, LINE_HEIGHT);
    ui.label(hms_rect, &hms_text)
        .font(ctx.font_large)
        .center()
        .draw();

    let menu_rect = hms_rect.offset(0, LINE_HEIGHT);
    ui.label(menu_rect, "[Click] to Menu").center().draw();
}
