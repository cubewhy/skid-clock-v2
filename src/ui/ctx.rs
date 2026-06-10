use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};
use u8g2_fonts::FontRenderer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEvent {
    Nothing,
    PrimaryUp,
    PrimaryDown,
    PrimaryLeft,
    PrimaryRight,
    PrimaryConfirm,
    Back,

    SecondaryUp,
    SecondaryRight,
    SecondaryLeft,
    SecondaryDown,
    SecondaryConfirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiInputConfig {
    pub confirm_action: UiEvent,
    pub up_action: UiEvent,
    pub down_action: UiEvent,
}

impl Default for UiInputConfig {
    fn default() -> Self {
        Self {
            confirm_action: UiEvent::PrimaryConfirm,
            up_action: UiEvent::PrimaryUp,
            down_action: UiEvent::PrimaryDown,
        }
    }
}

pub struct Ui<'a, D> {
    pub target: &'a mut D,
    pub font: &'a FontRenderer,
}

impl<'a, D> Ui<'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    pub fn new(target: &'a mut D, font: &'a FontRenderer) -> Self {
        Self { target, font }
    }
}
