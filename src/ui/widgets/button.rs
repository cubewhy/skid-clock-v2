use crate::ui::ctx::Ui;
use crate::ui::layout::Rect;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable},
};
use u8g2_fonts::types::{FontColor, HorizontalAlignment, VerticalPosition};

impl<'a, D> Ui<'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    pub fn button(&mut self, rect: Rect, text: &str, is_selected: bool) {
        let font_color = if is_selected {
            // Draw selected highlight box matching component Rect
            let style = PrimitiveStyleBuilder::new()
                .fill_color(BinaryColor::On)
                .build();
            Rectangle::new(
                Point::new(rect.x, rect.y),
                Size::new(rect.width, rect.height),
            )
            .draw_styled(&style, self.target)
            .ok();
            FontColor::Transparent(BinaryColor::Off)
        } else {
            FontColor::Transparent(BinaryColor::On)
        };

        self.font
            .render_aligned(
                text,
                Point::new(rect.x + 4, rect.y + rect.height as i32 - 3), // Centered padding
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                font_color,
                self.target,
            )
            .ok();
    }
}
