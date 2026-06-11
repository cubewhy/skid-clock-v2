use crate::{
    app_context::AppContext,
    apps::App,
    display::UnifiedDisplay,
    ui::{
        Rect, Ui,
        ctx::UiEvents,
        layout::{FlexDirection, FlexNode},
    },
};

pub fn update(event: UiEvents, selected_index: &mut i32) -> Option<App> {
    if event.contains(UiEvents::PRIMARY_UP) && *selected_index > 0 {
        *selected_index -= 1;
    }

    if event.contains(UiEvents::PRIMARY_DOWN) && *selected_index < 3 {
        *selected_index += 1;
    }

    if event.contains(UiEvents::PRIMARY_CONFIRM) {
        return (*selected_index == 0).then_some(App::Clock);
    }

    None
}

pub fn draw(ctx: &mut AppContext, selected_index: i32) {
    let resolution = ctx.display_1_3.resolution();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);
    let screen_rect = Rect::new(0, 0, resolution.width, resolution.height);

    let mut header_rect = Rect::default();
    let mut list_rect = Rect::default();

    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(screen_rect.width, 20)
                .assign_to(&mut header_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut list_rect),
        )
        .layout(screen_rect);

    ui.label(header_rect, "SYSTEM MENU").center().draw();

    let menu_items = ["Realtime Clock", "Time Tools", "Arcade Games", "Settings"];
    let item_height = 10;
    let visible_count = 4;

    ui.scroll_list(
        list_rect,
        &menu_items,
        selected_index.max(0) as usize,
        visible_count,
        item_height,
        |ui_ctx, item_rect, item_name, is_selected| {
            if is_selected {
                ui_ctx.label(item_rect, &format!("> {}", item_name)).draw();
            } else {
                ui_ctx.label(item_rect, &format!("  {}", item_name)).draw();
            }
        },
    );
}
