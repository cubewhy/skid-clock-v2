use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::UnifiedDisplay,
    ui::{
        Rect, Ui, UiEvents,
        layout::{AlignItems, FlexDirection, FlexNode},
    },
};
use embedded_graphics::pixelcolor::BinaryColor;
use rand::{RngExt, rngs::SmallRng};
use std::time::Instant;
use std::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FoodSize {
    Small,  // 1x1 grid, 2x2 pixels
    Medium, // 1x1 grid, 4x4 pixels
    Large,  // 2x2 grid, 8x8 pixels
}

pub struct SnakeState {
    body: Vec<(i16, i16)>, // Grid coordinates (x, y)
    direction: Direction,
    next_direction: Direction,
    food: (i16, i16), // Top-left grid coordinate of the food
    food_size: FoodSize,
    score: u32,
    is_game_over: bool,
    last_tick: Instant,
    rng: SmallRng,
}

impl Default for SnakeState {
    fn default() -> Self {
        let initial_body = vec![(5, 5), (4, 5), (3, 5)];
        let mut rng: SmallRng = rand::make_rng();

        let grid_width = 30;
        let grid_height = 11;

        let food_size = match rng.random_range(0..3) {
            0 => FoodSize::Small,
            1 => FoodSize::Medium,
            _ => FoodSize::Large,
        };

        // Ensure large food doesn't spawn out of bounds
        let (max_w, max_h) = match food_size {
            FoodSize::Large => (grid_width - 1, grid_height - 1),
            _ => (grid_width, grid_height),
        };

        let food = (rng.random_range(0..max_w), rng.random_range(0..max_h));

        Self {
            body: initial_body,
            direction: Direction::Right,
            next_direction: Direction::Right,
            food,
            food_size,
            score: 0,
            is_game_over: false,
            last_tick: Instant::now(),
            rng,
        }
    }
}

impl SnakeState {
    fn generate_food(&mut self, grid_w: i16, grid_h: i16) {
        // Randomize the size first to determine spatial constraints
        self.food_size = match self.rng.random_range(0..3) {
            0 => FoodSize::Small,
            1 => FoodSize::Medium,
            _ => FoodSize::Large,
        };

        let (max_w, max_h) = match self.food_size {
            FoodSize::Large => (grid_w - 1, grid_h - 1),
            _ => (grid_w, grid_h),
        };

        loop {
            let potential_food = (
                self.rng.random_range(0..max_w),
                self.rng.random_range(0..max_h),
            );

            // Verify if any segment of the food overlaps with the snake body
            let mut collides = false;
            match self.food_size {
                FoodSize::Large => {
                    for dx in 0..2 {
                        for dy in 0..2 {
                            if self
                                .body
                                .contains(&(potential_food.0 + dx, potential_food.1 + dy))
                            {
                                collides = true;
                            }
                        }
                    }
                }
                _ => {
                    if self.body.contains(&potential_food) {
                        collides = true;
                    }
                }
            }

            if !collides {
                self.food = potential_food;
                break;
            }
        }
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut SnakeState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    if state.is_game_over {
        if ctx.menu_events.contains(UiEvents::KEY_7) || ctx.menu_events.contains(UiEvents::CONFIRM)
        {
            *state = SnakeState::default();
        }
        return None;
    }

    if ctx.input_manager.is_down(UiEvents::UP) && state.direction != Direction::Down {
        state.next_direction = Direction::Up;
    } else if ctx.input_manager.is_down(UiEvents::DOWN) && state.direction != Direction::Up {
        state.next_direction = Direction::Down;
    } else if ctx.input_manager.is_down(UiEvents::LEFT) && state.direction != Direction::Right {
        state.next_direction = Direction::Left;
    } else if ctx.input_manager.is_down(UiEvents::RIGHT) && state.direction != Direction::Left {
        state.next_direction = Direction::Right;
    }

    let grid_width = 30;
    let grid_height = 11;

    // Core physics update loop running at 150ms intervals
    if state.last_tick.elapsed().as_millis() >= 150 {
        state.direction = state.next_direction;
        let head = state.body[0];

        let mut new_head = match state.direction {
            Direction::Up => (head.0, head.1 - 1),
            Direction::Down => (head.0, head.1 + 1),
            Direction::Left => (head.0 - 1, head.1),
            Direction::Right => (head.0 + 1, head.1),
        };

        new_head.0 = (new_head.0 + grid_width) % grid_width;
        new_head.1 = (new_head.1 + grid_height) % grid_height;

        if state.body.contains(&new_head) {
            state.is_game_over = true;
            return None;
        }

        // Insert new structural segment location
        state.body.insert(0, new_head);

        // Check food ingestion based on its multi-grid size footprint
        let ate_food = match state.food_size {
            FoodSize::Large => {
                new_head.0 >= state.food.0
                    && new_head.0 < state.food.0 + 2
                    && new_head.1 >= state.food.1
                    && new_head.1 < state.food.1 + 2
            }
            _ => new_head == state.food,
        };

        if ate_food {
            let points = match state.food_size {
                FoodSize::Small => 5,
                FoodSize::Medium => 10,
                FoodSize::Large => 20,
            };
            state.score += points;
            state.generate_food(grid_width, grid_height);
        } else {
            state.body.pop();
        }

        state.last_tick = Instant::now();
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &SnakeState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut rect_score = Rect::default();
    let mut rect_divider = Rect::default();
    let mut rect_board = Rect::default();

    let root = FlexNode::new(FlexDirection::Column)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 11)
                .assign_to(&mut rect_score),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 2)
                .assign_to(&mut rect_divider),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut rect_board),
        );

    root.layout(display_bounds);

    // Score Header
    let mut score_bytes = [0u8; 16];
    let score_str = msg_format(&mut score_bytes, "SCORE: ", state.score);
    ui.label(rect_score, score_str).center().draw();
    ui.horizontal_divider(rect_divider);

    // Game Board Core Parameters
    let cell_size = 4;
    let offset_x = rect_board.x + (rect_board.width as i32 - (30 * cell_size)) / 2;
    let offset_y = rect_board.y + (rect_board.height as i32 - (11 * cell_size)) / 2;

    // Render outer bounding wall border lines
    ui.draw_stroke_rect(
        Rect::new(
            offset_x - 1,
            offset_y - 1,
            (30 * cell_size) as u32 + 2,
            (11 * cell_size) as u32 + 2,
        ),
        BinaryColor::On,
        1,
    );

    // Draw active snake body segments
    for &(bx, by) in &state.body {
        ui.draw_filled_rect(
            Rect::new(
                offset_x + (bx as i32 * cell_size),
                offset_y + (by as i32 * cell_size),
                cell_size as u32,
                cell_size as u32,
            ),
            BinaryColor::On,
        );
    }

    // Render active food target with custom screen sizes
    let fx = offset_x + (state.food.0 as i32 * cell_size);
    let fy = offset_y + (state.food.1 as i32 * cell_size);

    match state.food_size {
        FoodSize::Small => {
            // Small Food: Draws a tiny 2x2 dot centered inside the 4x4 grid space
            ui.draw_filled_rect(Rect::new(fx + 1, fy + 1, 2, 2), BinaryColor::On);
        }
        FoodSize::Medium => {
            // Medium Food: Draws a normal 4x4 filled block completely filling 1 grid cell
            ui.draw_filled_rect(
                Rect::new(fx, fy, cell_size as u32, cell_size as u32),
                BinaryColor::On,
            );
        }
        FoodSize::Large => {
            // Large Food: Draws a massive 8x8 filled block covering a 2x2 grid tile cluster
            ui.draw_filled_rect(
                Rect::new(fx, fy, (cell_size * 2) as u32, (cell_size * 2) as u32),
                BinaryColor::On,
            );
        }
    }

    // Game Over Overlay Mask Window
    if state.is_game_over {
        ui.draw_filled_rect(rect_board, BinaryColor::Off);
        ui.draw_stroke_rect(rect_board, BinaryColor::On, 1);

        let mut rect_line1 = Rect::default();
        let mut rect_line2 = Rect::default();

        let over_layout = FlexNode::new(FlexDirection::Column)
            .align_items(AlignItems::Stretch)
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_flex(1)
                    .assign_to(&mut rect_line1),
            )
            .child(
                FlexNode::new(FlexDirection::Row)
                    .with_flex(1)
                    .assign_to(&mut rect_line2),
            );

        over_layout.layout(rect_board);

        ui.label(rect_line1, "GAME OVER!").center().draw();
        ui.label(rect_line2, "[7] Restart").center().draw();
    }
}

fn msg_format<'a>(buf: &'a mut [u8], prefix: &str, val: u32) -> &'a str {
    let p_len = prefix.len();
    buf[..p_len].copy_from_slice(prefix.as_bytes());
    let mut rem = val;
    let mut idx = buf.len() - 1;
    if rem == 0 {
        buf[p_len] = b'0';
        return core::str::from_utf8(&buf[..p_len + 1]).unwrap_or("");
    }
    while rem > 0 && idx >= p_len {
        buf[idx] = b'0' + (rem % 10) as u8;
        rem /= 10;
        idx -= 1;
    }
    let shift = idx + 1 - p_len;
    for i in p_len..buf.len() - shift {
        buf[i] = buf[i + shift];
    }
    core::str::from_utf8(&buf[..buf.len() - shift]).unwrap_or("")
}
