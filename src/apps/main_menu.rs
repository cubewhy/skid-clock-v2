use crate::{
    app_context::AppContext,
    apps::App,
    ui::{Ui, UiEvent},
};

pub fn update(event: UiEvent, selected_index: &mut i32) -> Option<App> {
    match event {
        UiEvent::PrimaryUp => {
            if *selected_index > 0 {
                *selected_index -= 1;
            }
        }
        UiEvent::PrimaryDown => {
            if *selected_index < 2 {
                *selected_index += 1;
            }
        }
        UiEvent::PrimaryConfirm => {
            return match selected_index {
                0 => Some(App::Clock),
                _ => None,
            };
        }
        _ => {}
    }
    None
}

pub fn draw(ctx: &mut AppContext, selected_index: i32) {}
