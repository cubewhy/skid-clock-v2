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
    pub fn checkbox(&mut self, rect: Rect, text: &str, checked: bool, is_selected: bool) {
        // Handle selection state by inverting colors and drawing a background block
        let fg_color = if is_selected {
            let style = PrimitiveStyleBuilder::new()
                .fill_color(BinaryColor::On)
                .build();
            Rectangle::new(
                Point::new(rect.x, rect.y),
                Size::new(rect.width, rect.height),
            )
            .draw_styled(&style, self.target)
            .ok();
            BinaryColor::Off
        } else {
            BinaryColor::On
        };

        // Box configuration (8x8 pixels, vertically centered)
        let box_size = 8;
        let box_y = rect.y + (rect.height as i32 - box_size) / 2;
        let box_x = rect.x + 4;

        // Draw the outer bounding box frame
        let box_style = PrimitiveStyleBuilder::new()
            .stroke_color(fg_color)
            .stroke_width(1)
            .build();
        Rectangle::new(
            Point::new(box_x, box_y),
            Size::new(box_size as u32, box_size as u32),
        )
        .draw_styled(&box_style, self.target)
        .ok();

        // Draw a solid inner mark if checked
        if checked {
            let mark_style = PrimitiveStyleBuilder::new().fill_color(fg_color).build();
            Rectangle::new(Point::new(box_x + 2, box_y + 2), Size::new(4, 4))
                .draw_styled(&mark_style, self.target)
                .ok();
        }

        // Render label text next to the checkbox frame
        self.font
            .render_aligned(
                text,
                Point::new(box_x + box_size + 6, rect.y + rect.height as i32 - 3),
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                FontColor::Transparent(fg_color),
                self.target,
            )
            .ok();
    }
}
