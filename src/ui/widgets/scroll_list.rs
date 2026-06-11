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
        let scrollbar_width = 4;
        let scrollbar_gap = 2;
        let reserved_space = scrollbar_width + scrollbar_gap;

        let item_width = rect.width.saturating_sub(reserved_space);

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
                item_width,
                item_height,
            );

            render_item(self, item_rect, item, is_selected);
        }

        let total_items = items.len();
        if total_items > visible_count {
            let scrollbar_x = rect.x + (rect.width - scrollbar_width) as i32;

            let thumb_height = (rect.height * visible_count as u32) / total_items as u32;
            let thumb_height = thumb_height.max(4);

            let remaining_track_height = rect.height.saturating_sub(thumb_height);
            let max_scroll_offset = total_items - visible_count;

            let thumb_y = rect.y
                + ((scroll_offset as u32 * remaining_track_height) / max_scroll_offset as u32)
                    as i32;

            let scrollbar_rect = Rect::new(scrollbar_x, thumb_y, scrollbar_width, thumb_height);

            self.draw_filled_rect(scrollbar_rect, BinaryColor::On);
        }
    }
}
