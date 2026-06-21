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

const MAP_COLS: usize = 21;
const MAP_ROWS: usize = 9;
const CELL_SIZE: i32 = 5;

const TILE_WALL: u8 = 0;
const TILE_DOT: u8 = 1;
const TILE_EMPTY: u8 = 2;
const TILE_BIG_DOT: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PacEntity {
    x: i8,
    y: i8,
    dx: i8,
    dy: i8,
}

pub struct PacmanState {
    maze: [[u8; MAP_COLS]; MAP_ROWS],
    pacman: PacEntity,
    ghosts: [PacEntity; 2],
    score: u32,
    lives: i8,
    total_dots: u32,
    is_game_over: bool,
    is_game_win: bool,
    last_tick: Instant,
    power_mode: bool,
    power_timer: u16,
    rng: SmallRng,
}

impl Default for PacmanState {
    fn default() -> Self {
        let rng: SmallRng = rand::make_rng();

        let mut state = Self {
            maze: [[TILE_WALL; MAP_COLS]; MAP_ROWS],
            pacman: PacEntity {
                x: 1,
                y: 1,
                dx: 1,
                dy: 0,
            },
            ghosts: [
                PacEntity {
                    x: (MAP_COLS - 2) as i8,
                    y: (MAP_ROWS - 2) as i8,
                    dx: -1,
                    dy: 0,
                }, // Red Ghost
                PacEntity {
                    x: 1,
                    y: (MAP_ROWS - 2) as i8,
                    dx: 0,
                    dy: -1,
                }, // Pink Ghost
            ],
            score: 0,
            lives: 3,
            total_dots: 0,
            is_game_over: false,
            is_game_win: false,
            last_tick: Instant::now(),
            power_mode: false,
            power_timer: 0,
            rng,
        };

        state.generate_pac_level();
        state
    }
}

impl PacmanState {
    fn carve_pac_maze(maze: &mut [[u8; MAP_COLS]; MAP_ROWS], r: i32, c: i32, rng: &mut SmallRng) {
        let mut dirs = [[0, 2], [0, -2], [2, 0], [-2, 0]];
        for i in 0..4 {
            let r_idx = rng.random_range(i..4);
            dirs.swap(i, r_idx);
        }
        for i in 0..4 {
            let next_r = r + dirs[i][0];
            let next_c = c + dirs[i][1];
            if next_r > 0
                && next_r < (MAP_ROWS - 1) as i32
                && next_c > 0
                && next_c < (MAP_COLS - 1) as i32
                && maze[next_r as usize][next_c as usize] == TILE_WALL
            {
                maze[next_r as usize][next_c as usize] = TILE_DOT;
                maze[(r + dirs[i][0] / 2) as usize][(c + dirs[i][1] / 2) as usize] = TILE_DOT;
                Self::carve_pac_maze(maze, next_r, next_c, rng);
            }
        }
    }

    fn generate_pac_level(&mut self) {
        self.total_dots = 0;
        self.maze = [[TILE_WALL; MAP_COLS]; MAP_ROWS];

        self.maze[1][1] = TILE_DOT;
        Self::carve_pac_maze(&mut self.maze, 1, 1, &mut self.rng);

        // Break walls randomly to create extra loops and pathways
        for r in 1..(MAP_ROWS - 1) {
            for c in 1..(MAP_COLS - 1) {
                if self.maze[r][c] == TILE_WALL && self.rng.random_range(0..100) < 22 {
                    self.maze[r][c] = TILE_DOT;
                }
            }
        }

        // Clear player/ghost spawn regions
        self.maze[1][1] = TILE_EMPTY;
        self.maze[MAP_ROWS - 2][MAP_COLS - 2] = TILE_EMPTY;
        self.maze[MAP_ROWS - 2][1] = TILE_EMPTY;

        // Overlay symmetric layout allocations for 4 power pellets
        let big_dot_coords = [
            [1, 3],
            [1, MAP_COLS - 4],
            [MAP_ROWS - 2, 3],
            [MAP_ROWS - 2, MAP_COLS - 4],
        ];
        for coord in &big_dot_coords {
            self.maze[coord[0]][coord[1]] = TILE_BIG_DOT;
        }

        // Rescan target target bounds calculations
        for r in 0..MAP_ROWS {
            for c in 0..MAP_COLS {
                if self.maze[r][c] == TILE_DOT || self.maze[r][c] == TILE_BIG_DOT {
                    self.total_dots += 1;
                }
            }
        }
    }
}

pub fn update(ctx: &mut UpdateContext, state: &mut PacmanState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    if state.is_game_over || state.is_game_win {
        if ctx.menu_events.contains(UiEvents::KEY_7) || ctx.menu_events.contains(UiEvents::CONFIRM)
        {
            *state = PacmanState::default();
        }
        return None;
    }

    // Directional orientation adjustments via keyboard event updates
    if ctx.input_manager.is_down(UiEvents::LEFT) {
        state.pacman.dx = -1;
        state.pacman.dy = 0;
    } else if ctx.input_manager.is_down(UiEvents::RIGHT) {
        state.pacman.dx = 1;
        state.pacman.dy = 0;
    } else if ctx.input_manager.is_down(UiEvents::UP) {
        state.pacman.dx = 0;
        state.pacman.dy = -1;
    } else if ctx.input_manager.is_down(UiEvents::DOWN) {
        state.pacman.dx = 0;
        state.pacman.dy = 1;
    }

    // Process structured physics cycles scaled exactly to 250ms interval gates
    if state.last_tick.elapsed().as_millis() >= 250 {
        state.last_tick = Instant::now();

        // Power Mode timer progression countdown rules
        if state.power_mode {
            if state.power_timer > 0 {
                state.power_timer -= 1;
            } else {
                state.power_mode = false;
            }
        }

        // 1. Advance Pacman kinematic translation offsets
        let next_x = state.pacman.x + state.pacman.dx;
        let next_y = state.pacman.y + state.pacman.dy;

        if next_x >= 0
            && next_x < MAP_COLS as i8
            && next_y >= 0
            && next_y < MAP_ROWS as i8
            && state.maze[next_y as usize][next_x as usize] != TILE_WALL
        {
            state.pacman.x = next_x;
            state.pacman.y = next_y;

            let current_tile = state.maze[state.pacman.y as usize][state.pacman.x as usize];
            if current_tile == TILE_DOT {
                state.maze[state.pacman.y as usize][state.pacman.x as usize] = TILE_EMPTY;
                state.score += 10;
                state.total_dots = state.total_dots.saturating_sub(1);
            } else if current_tile == TILE_BIG_DOT {
                state.maze[state.pacman.y as usize][state.pacman.x as usize] = TILE_EMPTY;
                state.score += 50;
                state.total_dots = state.total_dots.saturating_sub(1);
                state.power_mode = true;
                state.power_timer = 32; // 32 ticks * 250ms = 8 seconds total duration window
            }

            if state.total_dots == 0 {
                state.is_game_win = true;
                return None;
            }
        }

        // 2. Advance Ghost behavior loops pathfinding vectors
        let ghost_dirs = [[1, 0], [-1, 0], [0, 1], [0, -1]];

        for i in 0..2 {
            let mut g = state.ghosts[i];
            let mut best_dx = g.dx;
            let mut best_dy = g.dy;

            if i == 0 {
                // Ghost 0: Blinky AI (Target tracking chase/flee Manhattan metrics optimization)
                let mut target_dist = if state.power_mode { -1 } else { 999 };

                for ghost_dir in ghost_dirs {
                    let tx = g.x + ghost_dir[0];
                    let ty = g.y + ghost_dir[1];

                    if tx >= 0
                        && tx < MAP_COLS as i8
                        && ty >= 0
                        && ty < MAP_ROWS as i8
                        && state.maze[ty as usize][tx as usize] != TILE_WALL
                    {
                        // Enforce anti-reversing movement conditions
                        if ghost_dir[0] == -g.dx && ghost_dir[1] == -g.dy {
                            continue;
                        }

                        let dist = (tx as i32 - state.pacman.x as i32).abs()
                            + (ty as i32 - state.pacman.y as i32).abs();
                        if state.power_mode {
                            // Flee condition path layout optimization selection
                            if dist > target_dist {
                                target_dist = dist;
                                best_dx = ghost_dir[0];
                                best_dy = ghost_dir[1];
                            }
                        } else {
                            // Active pursuit interception target vector mapping optimization
                            if dist < target_dist {
                                target_dist = dist;
                                best_dx = ghost_dir[0];
                                best_dy = ghost_dir[1];
                            }
                        }
                    }
                }
            } else {
                // Ghost 1: Pinky AI (Stochastic exploratory random navigation profiles)
                let mut valid_dirs = [[0i8; 2]; 4];
                let mut valid_count = 0;

                for ghost_dir in ghost_dirs {
                    let tx = g.x + ghost_dir[0];
                    let ty = g.y + ghost_dir[1];

                    if tx >= 0
                        && tx < MAP_COLS as i8
                        && ty >= 0
                        && ty < MAP_ROWS as i8
                        && state.maze[ty as usize][tx as usize] != TILE_WALL
                    {
                        if ghost_dir[0] == -g.dx && ghost_dir[1] == -g.dy {
                            continue;
                        }
                        valid_dirs[valid_count] = [ghost_dir[0], ghost_dir[1]];
                        valid_count += 1;
                    }
                }

                if valid_count > 0
                    && ((g.dx == 0 && g.dy == 0)
                        || state.power_mode
                        || state.rng.random_range(0..100) < 35)
                {
                    let rd = state.rng.random_range(0..valid_count);
                    best_dx = valid_dirs[rd][0];
                    best_dy = valid_dirs[rd][1];
                }
            }

            // Step translation validation tracking routines
            let dest_x = g.x + best_dx;
            let dest_y = g.y + best_dy;

            if dest_x >= 0
                && dest_x < MAP_COLS as i8
                && dest_y >= 0
                && dest_y < MAP_ROWS as i8
                && state.maze[dest_y as usize][dest_x as usize] != TILE_WALL
            {
                g.dx = best_dx;
                g.dy = best_dy;
                g.x += g.dx;
                g.y += g.dy;
            } else {
                // Simple wall bounce backup logic parameter rules
                g.dx = -g.dx;
                g.dy = -g.dy;
            }
            state.ghosts[i] = g;

            // 3. Bi-directional entity conflict intersection loops resolution
            if state.ghosts[i].x == state.pacman.x && state.ghosts[i].y == state.pacman.y {
                if state.power_mode {
                    // Consume ghost configuration parameters criteria
                    state.score += 200;
                    if i == 0 {
                        state.ghosts[0] = PacEntity {
                            x: (MAP_COLS - 2) as i8,
                            y: (MAP_ROWS - 2) as i8,
                            dx: -1,
                            dy: 0,
                        };
                    } else {
                        state.ghosts[1] = PacEntity {
                            x: 1,
                            y: (MAP_ROWS - 2) as i8,
                            dx: 0,
                            dy: -1,
                        };
                    }
                } else {
                    // Standard damage calculation terminal execution tree
                    state.lives -= 1;
                    if state.lives <= 0 {
                        state.is_game_over = true;
                        return None;
                    }
                    // Reset global stage matrix layout configuration components positioning metadata
                    state.pacman = PacEntity {
                        x: 1,
                        y: 1,
                        dx: 1,
                        dy: 0,
                    };
                    state.ghosts[0] = PacEntity {
                        x: (MAP_COLS - 2) as i8,
                        y: (MAP_ROWS - 2) as i8,
                        dx: -1,
                        dy: 0,
                    };
                    state.ghosts[1] = PacEntity {
                        x: 1,
                        y: (MAP_ROWS - 2) as i8,
                        dx: 0,
                        dy: -1,
                    };
                    break;
                }
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &PacmanState) {
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut rect_stats = Rect::default();
    let mut rect_divider = Rect::default();
    let mut rect_board = Rect::default();

    let root = FlexNode::new(FlexDirection::Column)
        .align_items(AlignItems::Stretch)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 11)
                .assign_to(&mut rect_stats),
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

    // Render Stats Headers Information Array Rows
    let mut score_bytes = [0u8; 16];
    let score_str = msg_format(&mut score_bytes, "SCORE:", state.score);
    ui.label(Rect::new(2, 0, 56, 11), score_str).draw();

    // Toggle conditional blinkers indicators metrics matching active context configurations
    if state.power_mode && (state.power_timer > 8 || (state_millis_now() / 150).is_multiple_of(2)) {
        ui.label(Rect::new(60, 0, 42, 11), "[POWER]").draw();
    }

    let mut lives_bytes = [0u8; 8];
    let lives_str = msg_format(&mut lives_bytes, "V:", state.lives as u32);
    ui.label(Rect::new(105, 0, 20, 11), lives_str).draw();

    ui.divider(rect_divider);

    // Map offset positioning vectors variables
    let start_x = rect_board.x + 11;
    let start_y = rect_board.y + 3;

    // Output structural maze segments matrix coordinates configurations parameters elements
    for r in 0..MAP_ROWS {
        for c in 0..MAP_COLS {
            let bx = start_x + c as i32 * CELL_SIZE;
            let by = start_y + r as i32 * CELL_SIZE;

            match state.maze[r][c] {
                TILE_WALL => {
                    ui.draw_filled_rect(
                        Rect::new(
                            bx + 1,
                            by + 1,
                            (CELL_SIZE - 1) as u32,
                            (CELL_SIZE - 1) as u32,
                        ),
                        BinaryColor::On,
                    );
                }
                TILE_DOT => {
                    ui.draw_filled_rect(
                        Rect::new(bx + CELL_SIZE / 2, by + CELL_SIZE / 2, 1, 1),
                        BinaryColor::On,
                    );
                }
                TILE_BIG_DOT => {
                    ui.draw_filled_rect(Rect::new(bx + 1, by + 1, 3, 3), BinaryColor::On);
                }
                _ => {}
            }
        }
    }

    // Render Player Avatar Entity representation graphics configurations (Pacman structure profile)
    let px = start_x + state.pacman.x as i32 * CELL_SIZE + CELL_SIZE / 2;
    let py = start_y + state.pacman.y as i32 * CELL_SIZE + CELL_SIZE / 2;
    ui.draw_stroke_rect(Rect::new(px - 2, py - 2, 5, 5), BinaryColor::On, 1);

    // Clear orientation tracking slice vectors to mask directional open mouth gaps
    let mouth_rect = if state.pacman.dx == 1 {
        Rect::new(px + 2, py, 1, 1)
    } else if state.pacman.dx == -1 {
        Rect::new(px - 2, py, 1, 1)
    } else if state.pacman.dy == 1 {
        Rect::new(px, py + 2, 1, 1)
    } else {
        Rect::new(px, py - 2, 1, 1)
    };
    ui.draw_filled_rect(mouth_rect, BinaryColor::Off);

    // Render Enemy Entity structures matrix sets variables layouts channels properties
    for i in 0..2 {
        let gx = start_x + state.ghosts[i].x as i32 * CELL_SIZE;
        let gy = start_y + state.ghosts[i].y as i32 * CELL_SIZE;

        if state.power_mode {
            // Under Frightened status criteria trigger flashing tracking cross shapes parameters overlays
            if state.power_timer > 8 || (state_millis_now() / 200).is_multiple_of(2) {
                ui.draw_filled_rect(Rect::new(gx + 1, gy + 2, 3, 1), BinaryColor::On);
                ui.draw_filled_rect(Rect::new(gx + 2, gy + 1, 1, 3), BinaryColor::On);
            } else {
                ui.draw_stroke_rect(
                    Rect::new(gx + 1, gy, (CELL_SIZE - 1) as u32, (CELL_SIZE - 1) as u32),
                    BinaryColor::On,
                    1,
                );
            }
        } else {
            // Normal system behavior profiling execution blocks (Solid inner blocks vs hollow wrappers metrics)
            ui.draw_stroke_rect(
                Rect::new(gx + 1, gy, (CELL_SIZE - 1) as u32, (CELL_SIZE - 1) as u32),
                BinaryColor::On,
                1,
            );
            if i == 1 {
                ui.draw_filled_rect(
                    Rect::new(
                        gx + 2,
                        gy + 1,
                        (CELL_SIZE - 3) as u32,
                        (CELL_SIZE - 3) as u32,
                    ),
                    BinaryColor::On,
                );
            }
        }
    }

    // === Game Over / Terminated Victory Modal dialog window frames overlays parameters ===
    if state.is_game_over || state.is_game_win {
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

        if state.is_game_win {
            ui.label(rect_line1, "YOU WIN!").center().draw();
        } else {
            ui.label(rect_line1, "GAME OVER").center().draw();
        }
        ui.label(rect_line2, "[7] Replay").center().draw();
    }
}

fn state_millis_now() -> u128 {
    chrono::Utc::now().timestamp_millis() as u128
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
        if idx == p_len {
            break;
        }
        idx -= 1;
    }
    let shift = idx + 1 - p_len;
    for i in p_len..buf.len() - shift {
        buf[i] = buf[i + shift];
    }
    core::str::from_utf8(&buf[..buf.len() - shift]).unwrap_or("")
}
