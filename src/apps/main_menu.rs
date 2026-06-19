use embedded_graphics::pixelcolor::BinaryColor;

use crate::{
    app_context::{AppContext, UpdateContext},
    apps::{App, settings::SettingsState},
    display::UnifiedDisplay,
    ui::{
        Rect, Ui,
        ctx::UiEvents,
        layout::{FlexDirection, FlexNode},
    },
};

pub fn update(ctx: &UpdateContext, selected_index: &mut i32) -> Option<App> {
    let events = ctx.menu_events;
    if events.contains(UiEvents::UP) {
        if *selected_index > 0 {
            *selected_index -= 1;
        } else {
            *selected_index = 3; // MAX INDEX
        }
    }

    if events.contains(UiEvents::DOWN) {
        if *selected_index < 3 {
            *selected_index += 1;
        } else {
            *selected_index = 0;
        }
    }

    if events.intersects(UiEvents::KEY_ESC | UiEvents::LEFT) {
        return Some(App::Clock);
    }

    if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_3 | UiEvents::RIGHT) {
        let app = match selected_index {
            0 => App::Clock,
            1 => App::TimeToolsMenu,
            2 => App::GamesMenu,
            3 => App::Settings(SettingsState::default()),
            _ => return None,
        };
        return Some(app);
    }

    None
}

pub fn draw(ctx: &mut AppContext, selected_index: i32) {
    let resolution = ctx.display_1_3.resolution();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);
    let screen_rect = Rect::new(0, 0, resolution.width, resolution.height);

    let mut header_rect = Rect::default();
    let mut line_rect = Rect::default();
    let mut bottom_line_rect = Rect::default();
    let mut list_rect = Rect::default();
    let mut footer_rect = Rect::default();

    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(screen_rect.width, 15)
                .assign_to(&mut header_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(screen_rect.width, 2)
                .assign_to(&mut line_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut list_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(screen_rect.width, 1)
                .assign_to(&mut bottom_line_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(screen_rect.width, 14)
                .assign_to(&mut footer_rect),
        )
        .layout(screen_rect);

    ui.label(header_rect, "SYSTEM MENU").center().draw();

    let start_point = (line_rect.x, line_rect.y);
    let end_point = (line_rect.x + line_rect.width as i32, line_rect.y);
    ui.draw_line(start_point, end_point, BinaryColor::On);

    let menu_items = ["Realtime Clock", "Time Tools", "Arcade Games", "Settings"];
    let item_height = 10;
    let visible_count = 3;

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

    ui.draw_line(
        (bottom_line_rect.x, bottom_line_rect.y),
        (
            bottom_line_rect.x + bottom_line_rect.width as i32,
            bottom_line_rect.y,
        ),
        BinaryColor::On,
    );

    ui.label(footer_rect, "cubewhy/skid-clock-v2")
        .center()
        .draw();
}
