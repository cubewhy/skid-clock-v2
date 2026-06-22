use bitflags::Flags;

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

pub fn update(ctx: &UpdateContext) -> Option<App> {
    let trigger_mask = UiEvents::all_named();

    ctx.menu_events
        .intersects(trigger_mask)
        .then_some(App::main_menu())
}

pub fn draw(ctx: &mut AppContext) {
    let now = chrono::Local::now();
    let ymd_text = now.format("%Y-%m-%d").to_string();
    let hms_text = now.format("%H:%M:%S").to_string();

    let bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut date_rect = Rect::default();
    let mut hms_rect = Rect::default();
    let mut menu_rect = Rect::default();

    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Column)
                .with_flex(1)
                .assign_to(&mut date_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Column)
                .with_flex(1)
                .assign_to(&mut hms_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Column)
                .with_flex(1)
                .assign_to(&mut menu_rect),
        )
        .layout(Rect::new(0, 5, bounds.width, 60));

    // Handle Network Signal Strength Bars
    if ctx.network.is_connected() {
        let rssi = ctx.network.get_rssi().unwrap_or(-100);

        let bar_count = if rssi >= -67 {
            3
        } else if rssi >= -80 {
            2
        } else {
            1
        };

        if bar_count >= 1 {
            ui.vertical_divider(Rect::new(bounds.width as i32 - 14, 13, 2, 4));
        }
        if bar_count >= 2 {
            ui.vertical_divider(Rect::new(bounds.width as i32 - 10, 10, 2, 7));
        }
        if bar_count >= 3 {
            ui.vertical_divider(Rect::new(bounds.width as i32 - 6, 7, 2, 10));
        }
    } else {
        let disconnect_rect = Rect::new(bounds.width as i32 - 14, 5, 12, 12);
        ui.label(disconnect_rect, "x").draw();
    }

    // Render Text UI Elements
    ui.label(date_rect, &ymd_text).center().draw();

    ui.label(hms_rect, &hms_text)
        .font(ctx.font_large)
        .center()
        .draw();

    ui.label(menu_rect, "[Click] to Menu").center().draw();
}
