use crate::ui::ctx::Ui;
use crate::ui::layout::Rect;
use embedded_graphics::{draw_target::DrawTarget, geometry::Point, pixelcolor::BinaryColor};
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

    pub fn draw(&mut self) {
        let font = self.font.unwrap_or(self.ui.font);

        let x = match self.align_h {
            HorizontalAlignment::Left => self.rect.x,
            HorizontalAlignment::Center => self.rect.x + (self.rect.width / 2) as i32,
            HorizontalAlignment::Right => self.rect.x + self.rect.width as i32,
        };

        let y = match self.align_v {
            VerticalPosition::Top => self.rect.y,
            VerticalPosition::Center => self.rect.y + (self.rect.height / 2) as i32,
            VerticalPosition::Bottom => self.rect.y + self.rect.height as i32,
            VerticalPosition::Baseline => self.rect.y,
        };

        font.render_aligned(
            self.text,
            Point::new(x, y),
            self.align_v,
            self.align_h,
            FontColor::Transparent(BinaryColor::On),
            self.ui.target,
        )
        .ok();
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
