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
    pub fn progress_bar(&mut self, rect: Rect, label_text: &str, progress: f32) {
        // Draw label text at the top of the component rect
        self.font
            .render_aligned(
                label_text,
                Point::new(rect.x, rect.y + 11),
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                FontColor::Transparent(BinaryColor::On),
                self.target,
            )
            .ok();

        // Calculate and draw progress border bar below the label
        let bar_y = rect.y + 14;
        let border_style = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(1)
            .build();
        Rectangle::new(Point::new(rect.x, bar_y), Size::new(rect.width, 6))
            .draw_styled(&border_style, self.target)
            .ok();

        let max_fill_width = rect.width.saturating_sub(2);
        let filled_width = (max_fill_width as f32 * progress.clamp(0.0, 1.0)) as u32;
        if filled_width > 0 {
            let fill_style = PrimitiveStyleBuilder::new()
                .fill_color(BinaryColor::On)
                .build();
            Rectangle::new(
                Point::new(rect.x + 1, bar_y + 1),
                Size::new(filled_width, 4),
            )
            .draw_styled(&fill_style, self.target)
            .ok();
        }
    }
}
