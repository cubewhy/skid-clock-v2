use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};
use u8g2_fonts::FontRenderer;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UiEvents: u16 {
        const PRIMARY_UP         = 1 << 0;  // 0x0001
        const PRIMARY_DOWN       = 1 << 1;  // 0x0002
        const PRIMARY_LEFT       = 1 << 2;  // 0x0004
        const PRIMARY_RIGHT      = 1 << 3;  // 0x0008
        const PRIMARY_CONFIRM    = 1 << 4;  // 0x0010
        const BACK               = 1 << 5;  // 0x0020

        const SECONDARY_UP       = 1 << 6;  // 0x0040
        const SECONDARY_DOWN     = 1 << 7;  // 0x0080
        const SECONDARY_LEFT     = 1 << 8;  // 0x0100
        const SECONDARY_RIGHT    = 1 << 9;  // 0x0200
        const SECONDARY_CONFIRM  = 1 << 10; // 0x0400
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
