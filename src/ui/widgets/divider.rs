use crate::ui::ctx::Ui;
use crate::ui::layout::Rect;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
};

impl<'a, D> Ui<'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    /// Draws a thin 1-pixel horizontal line divider centered inside the bounding box.
    pub fn horizontal_divider(&mut self, rect: Rect) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(1)
            .build();

        let center_y = rect.y + (rect.height as i32 / 2);

        Rectangle::new(Point::new(rect.x, center_y), Size::new(rect.width, 1))
            .draw_styled(&style, self.target)
            .ok();
    }

    /// Draws a thin 1-pixel vertical line divider centered inside the bounding box.
    pub fn vertical_divider(&mut self, rect: Rect) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(1)
            .build();

        // Calculate the horizontal center of the bounding box
        let center_x = rect.x + (rect.width as i32 / 2);

        // Draw a vertical line: 1-pixel wide, full rect height
        Rectangle::new(Point::new(center_x, rect.y), Size::new(1, rect.height))
            .draw_styled(&style, self.target)
            .ok();
    }
}
