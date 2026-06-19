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
        const KEY_ESC   = 1 << 0;
        const KEY_1     = 1 << 1;
        const KEY_2     = 1 << 2;
        const KEY_3     = 1 << 3;
        const KEY_4     = 1 << 4;
        const KEY_5     = 1 << 5;
        const KEY_6     = 1 << 6;
        const KEY_7     = 1 << 7; // also KEY_CONFIRM

        const UP        = 1 << 8;
        const DOWN      = 1 << 9;
        const LEFT      = 1 << 10;
        const RIGHT     = 1 << 11;
        const CONFIRM   = 1 << 12;
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
