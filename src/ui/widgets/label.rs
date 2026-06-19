use crate::ui::ctx::Ui;
use crate::ui::layout::Rect;
use embedded_graphics::{
    draw_target::{DrawTarget, DrawTargetExt},
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::Rectangle,
};
use u8g2_fonts::types::{FontColor, HorizontalAlignment, VerticalPosition};

pub struct LabelBuilder<'b, 'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    ui: &'b mut Ui<'a, D>,
    rect: Rect,
    text: &'b str,
    font: Option<&'a u8g2_fonts::FontRenderer>,
    align_h: HorizontalAlignment,
    align_v: VerticalPosition,
    scroll: bool,
    tick: u32,
    speed: u32,
}

impl<'b, 'a, D> LabelBuilder<'b, 'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    pub fn new(ui: &'b mut Ui<'a, D>, rect: Rect, text: &'b str) -> Self {
        Self {
            ui,
            rect,
            text,
            font: None,
            align_h: HorizontalAlignment::Left,
            align_v: VerticalPosition::Top,
            scroll: false,
            tick: 0,
            speed: 1,
        }
    }

    pub fn font(mut self, font: &'a u8g2_fonts::FontRenderer) -> Self {
        self.font = Some(font);
        self
    }

    pub fn center(mut self) -> Self {
        self.align_h = HorizontalAlignment::Center;
        self.align_v = VerticalPosition::Center;
        self
    }

    pub fn align(mut self, h: HorizontalAlignment, v: VerticalPosition) -> Self {
        self.align_h = h;
        self.align_v = v;
        self
    }

    /// Enables seamless continuous marquee scrolling.
    /// - `tick`: A monotonic frame counter from your main loop.
    /// - `speed`: Pixels to shift per tick (keep it low, e.g., 1 for maximum smoothness).
    pub fn scroll(mut self, tick: u32, speed: u32) -> Self {
        self.scroll = true;
        self.tick = tick;
        self.speed = speed;
        self
    }

    pub fn draw(&mut self) {
        let font = self.font.unwrap_or(self.ui.font);

        // Get total string width in pixels
        let text_width = font
            .get_rendered_dimensions(self.text, Point::zero(), VerticalPosition::Baseline)
            .ok()
            .and_then(|dims| dims.bounding_box)
            .map_or(0, |bbox| bbox.size.width as i32);

        // Create a cropped target restricted to the bounding rect
        let clip_rect = Rectangle::new(
            Point::new(self.rect.x, self.rect.y),
            Size::new(self.rect.width, self.rect.height),
        );
        let mut cropped_target = self.ui.target.cropped(&clip_rect);

        // Calculate Y alignment relative to the cropped origin
        let y = match self.align_v {
            VerticalPosition::Top => 0,
            VerticalPosition::Center => (self.rect.height / 2) as i32,
            VerticalPosition::Bottom => self.rect.height as i32,
            VerticalPosition::Baseline => 0,
        };

        // Continuous scrolling logic
        if self.scroll && text_width > self.rect.width as i32 {
            let gap = 32; // Pixel distance between the trailing text and the next loop
            let total_cycle = text_width + gap;

            // Linear, non-stopping pixel offset calculation
            let scroll_shift = ((self.tick * self.speed) as i32) % total_cycle;

            // Render 1st instance (moving out to the left)
            let x1 = -scroll_shift;
            font.render_aligned(
                self.text,
                Point::new(x1, y),
                self.align_v,
                HorizontalAlignment::Left,
                FontColor::Transparent(BinaryColor::On),
                &mut cropped_target,
            )
            .ok();

            // Render 2nd instance (wrapping around and entering from the right)
            let x2 = -scroll_shift + total_cycle;
            font.render_aligned(
                self.text,
                Point::new(x2, y),
                self.align_v,
                HorizontalAlignment::Left,
                FontColor::Transparent(BinaryColor::On),
                &mut cropped_target,
            )
            .ok();
        } else {
            // Fallback static rendering path when text fits or scrolling is disabled
            let x = match self.align_h {
                HorizontalAlignment::Left => 0,
                HorizontalAlignment::Center => (self.rect.width / 2) as i32,
                HorizontalAlignment::Right => self.rect.width as i32,
            };

            font.render_aligned(
                self.text,
                Point::new(x, y),
                self.align_v,
                self.align_h,
                FontColor::Transparent(BinaryColor::On),
                &mut cropped_target,
            )
            .ok();
        }
    }
}

impl<'a, D> Ui<'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    pub fn label<'b>(&'b mut self, rect: Rect, text: &'b str) -> LabelBuilder<'b, 'a, D> {
        LabelBuilder::new(self, rect, text)
    }
}
