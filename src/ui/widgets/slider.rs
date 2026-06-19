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
    /// Renders an interactive slider.
    /// - `value`: Position of the handle clamped between 0.0 and 1.0.
    pub fn slider(&mut self, rect: Rect, label_text: &str, value: f32, is_selected: bool) {
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

        // Draw the slider label at the top of the rect boundary
        self.font
            .render_aligned(
                label_text,
                Point::new(rect.x + 2, rect.y + 11),
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                FontColor::Transparent(fg_color),
                self.target,
            )
            .ok();

        // Calculate geometry for track line and active indicator handle
        let bar_y = rect.y + 16;
        let line_padding = 4;
        let track_width = rect.width.saturating_sub(line_padding * 2);

        // Draw horizontal track track line
        let line_style = PrimitiveStyleBuilder::new()
            .stroke_color(fg_color)
            .stroke_width(1)
            .build();
        Rectangle::new(
            Point::new(rect.x + line_padding as i32, bar_y),
            Size::new(track_width, 1),
        )
        .draw_styled(&line_style, self.target)
        .ok();

        // Compute indicator handle positioning (width 4, height 5) centered on the track line
        let handle_width = 4;
        let max_travel = track_width.saturating_sub(handle_width);
        let handle_x_offset = (max_travel as f32 * value.clamp(0.0, 1.0)) as i32;

        let handle_style = PrimitiveStyleBuilder::new().fill_color(fg_color).build();
        Rectangle::new(
            Point::new(rect.x + line_padding as i32 + handle_x_offset, bar_y - 2),
            Size::new(handle_width, 5),
        )
        .draw_styled(&handle_style, self.target)
        .ok();
    }
}
