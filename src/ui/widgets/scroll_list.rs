use crate::ui::ctx::Ui;
use crate::ui::layout::Rect;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

impl<'a, D> Ui<'a, D>
where
    D: DrawTarget<Color = BinaryColor>,
{
    pub fn scroll_list<T, F>(
        &mut self,
        rect: Rect, // Bounding box for the overall list container
        items: &[T],
        selected_index: usize,
        visible_count: usize,
        item_height: u32,
        mut render_item: F,
    ) where
        F: FnMut(&mut Self, Rect, &T, bool),
    {
        let scroll_offset = if selected_index >= visible_count {
            selected_index - visible_count + 1
        } else {
            0
        };

        for (i, item) in items
            .iter()
            .skip(scroll_offset)
            .take(visible_count)
            .enumerate()
        {
            let actual_index = i + scroll_offset;
            let is_selected = actual_index == selected_index;

            // Compute precise bounding box for this list item
            let item_rect = Rect::new(
                rect.x,
                rect.y + (i as i32 * item_height as i32),
                rect.width,
                item_height,
            );

            render_item(self, item_rect, item, is_selected);
        }
    }
}
