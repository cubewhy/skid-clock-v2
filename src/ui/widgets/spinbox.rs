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
    /// Renders a value adjusting element displaying: Label             < Value >
    pub fn spinbox(&mut self, rect: Rect, label_text: &str, value_text: &str, is_selected: bool) {
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

        let baseline_y = rect.y + rect.height as i32 - 3;

        // Render description label on the left side
        self.font
            .render_aligned(
                label_text,
                Point::new(rect.x + 4, baseline_y),
                VerticalPosition::Baseline,
                HorizontalAlignment::Left,
                FontColor::Transparent(fg_color),
                self.target,
            )
            .ok();

        // Right-to-Left step layout calculation for dependency-free string grouping
        let right_edge_x = rect.x + rect.width as i32 - 4;

        // 1. Draw right increment bracket '>'
        self.font
            .render_aligned(
                ">",
                Point::new(right_edge_x, baseline_y),
                VerticalPosition::Baseline,
                HorizontalAlignment::Right,
                FontColor::Transparent(fg_color),
                self.target,
            )
            .ok();

        // 2. Draw active inner value text (offset left of the bracket)
        let value_x = right_edge_x - 8;
        self.font
            .render_aligned(
                value_text,
                Point::new(value_x, baseline_y),
                VerticalPosition::Baseline,
                HorizontalAlignment::Right,
                FontColor::Transparent(fg_color),
                self.target,
            )
            .ok();

        // Measure dynamic string runtime dimension to offset decrement bracket cleanly
        let value_width = self
            .font
            .get_rendered_dimensions(value_text, Point::zero(), VerticalPosition::Baseline)
            .ok()
            .and_then(|dims| dims.bounding_box)
            .map_or(0, |bbox| bbox.size.width as i32);

        // 3. Draw left decrement bracket '<'
        let left_bracket_x = value_x - value_width - 4;
        self.font
            .render_aligned(
                "<",
                Point::new(left_bracket_x, baseline_y),
                VerticalPosition::Baseline,
                HorizontalAlignment::Right,
                FontColor::Transparent(fg_color),
                self.target,
            )
            .ok();
    }
}
