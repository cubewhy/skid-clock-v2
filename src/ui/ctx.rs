use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{Line, Primitive, PrimitiveStyle, Rectangle},
};
use u8g2_fonts::FontRenderer;

use bitflags::bitflags;

use crate::ui::Rect;

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

    /// Draw an rect (without fill)
    pub fn draw_stroke_rect(&mut self, rect: Rect, color: BinaryColor, stroke_width: u32) {
        let style = PrimitiveStyle::with_stroke(color, stroke_width);

        let eg_rect = Rectangle::new(
            Point::new(rect.x, rect.y),
            Size::new(rect.width, rect.height),
        )
        .into_styled(style);

        let _ = eg_rect.draw(self.target);
    }

    /// Draw a filled rect
    pub fn draw_filled_rect(&mut self, rect: Rect, color: BinaryColor) {
        let style = PrimitiveStyle::with_fill(color);

        let eg_rect = Rectangle::new(
            Point::new(rect.x, rect.y),
            Size::new(rect.width, rect.height),
        )
        .into_styled(style);

        let _ = eg_rect.draw(self.target);
    }

    /// Draw a line
    pub fn draw_line(&mut self, start: (i32, i32), end: (i32, i32), color: BinaryColor) {
        let style = PrimitiveStyle::with_stroke(color, 1);

        let eg_line =
            Line::new(Point::new(start.0, start.1), Point::new(end.0, end.1)).into_styled(style);

        let _ = eg_line.draw(self.target);
    }
}
