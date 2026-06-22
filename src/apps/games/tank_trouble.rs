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

const MAX_TRBL_BULLETS: usize = 8;

#[derive(Debug, Clone, Copy)]
struct TroubleBullet {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    bounces_left: i8,
    from_enemy: bool,
    active: bool,
}

impl Default for TroubleBullet {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            bounces_left: 0,
            from_enemy: false,
            active: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TroubleEnemy {
    x: f32,
    y: f32,
    target_x: f32,
    target_y: f32,
    turret_angle: f32,
    sweep_dir: f32,
    last_shot: Instant,
    last_x: f32,
    last_y: f32,
    stuck_frames: u8,
    active: bool,
}

#[derive(Debug, Clone, Copy)]
struct MazeWall {
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
    is_vertical: bool,
}

pub struct TankTroubleState {
    walls: Vec<MazeWall>,
    bullets: [TroubleBullet; MAX_TRBL_BULLETS],
    enemy: TroubleEnemy,
    player_x: f32,
    player_y: f32,
    player_angle: f32,
    score: u32,
    is_game_over: bool,
    last_tick: Instant,
    last_player_fire: Instant,
    ai_detour_ticks: u8,
    rng: SmallRng,
}

impl Default for TankTroubleState {
    fn default() -> Self {
        let rng: SmallRng = rand::make_rng();

        let mut state = Self {
            walls: Vec::with_capacity(14),
            bullets: [TroubleBullet::default(); MAX_TRBL_BULLETS],
            enemy: TroubleEnemy {
                x: 0.0,
                y: 0.0,
                target_x: 0.0,
                target_y: 0.0,
                turret_angle: 0.0,
                sweep_dir: 1.0,
                last_shot: Instant::now(),
                last_x: 0.0,
                last_y: 0.0,
                stuck_frames: 0,
                active: false,
            },
            player_x: 11.0,
            player_y: 52.0,
            player_angle: 0.0,
            score: 0,
            is_game_over: false,
            last_tick: Instant::now(),
            last_player_fire: Instant::now(),
            ai_detour_ticks: 0,
            rng,
        };

        state.generate_random_maze();
        state.respawn_enemy();
        state
    }
}

impl TankTroubleState {
    fn generate_random_maze(&mut self) {
        self.walls.clear();

        // 1. 注入 4 面边界大围墙
        self.walls.push(MazeWall {
            x1: 0,
            y1: 10,
            x2: 127,
            y2: 10,
            is_vertical: false,
        });
        self.walls.push(MazeWall {
            x1: 0,
            y1: 63,
            x2: 127,
            y2: 63,
            is_vertical: false,
        });
        self.walls.push(MazeWall {
            x1: 0,
            y1: 10,
            x2: 0,
            y2: 63,
            is_vertical: true,
        });
        self.walls.push(MazeWall {
            x1: 127,
            y1: 10,
            x2: 127,
            y2: 63,
            is_vertical: true,
        });

        // 2. 加载隔断墙阵列样式
        let style = self.rng.random_range(0..3);
        if style == 0 {
            self.walls.push(MazeWall {
                x1: 28,
                y1: 10,
                x2: 28,
                y2: 32,
                is_vertical: true,
            });
            self.walls.push(MazeWall {
                x1: 28,
                y1: 32,
                x2: 48,
                y2: 32,
                is_vertical: false,
            });
            self.walls.push(MazeWall {
                x1: 100,
                y1: 10,
                x2: 100,
                y2: 32,
                is_vertical: true,
            });
            self.walls.push(MazeWall {
                x1: 80,
                y1: 32,
                x2: 100,
                y2: 32,
                is_vertical: false,
            });
            self.walls.push(MazeWall {
                x1: 28,
                y1: 44,
                x2: 28,
                y2: 63,
                is_vertical: true,
            });
            self.walls.push(MazeWall {
                x1: 100,
                y1: 44,
                x2: 100,
                y2: 63,
                is_vertical: true,
            });
            self.walls.push(MazeWall {
                x1: 48,
                y1: 48,
                x2: 80,
                y2: 48,
                is_vertical: false,
            });
        } else if style == 1 {
            self.walls.push(MazeWall {
                x1: 24,
                y1: 24,
                x2: 54,
                y2: 24,
                is_vertical: false,
            });
            self.walls.push(MazeWall {
                x1: 74,
                y1: 24,
                x2: 104,
                y2: 24,
                is_vertical: false,
            });
            self.walls.push(MazeWall {
                x1: 24,
                y1: 50,
                x2: 54,
                y2: 50,
                is_vertical: false,
            });
            self.walls.push(MazeWall {
                x1: 74,
                y1: 50,
                x2: 104,
                y2: 50,
                is_vertical: false,
            });
            self.walls.push(MazeWall {
                x1: 40,
                y1: 24,
                x2: 40,
                y2: 50,
                is_vertical: true,
            });
            self.walls.push(MazeWall {
                x1: 88,
                y1: 24,
                x2: 88,
                y2: 50,
                is_vertical: true,
            });
        } else {
            self.walls.push(MazeWall {
                x1: 22,
                y1: 10,
                x2: 22,
                y2: 46,
                is_vertical: true,
            });
            self.walls.push(MazeWall {
                x1: 106,
                y1: 10,
                x2: 106,
                y2: 46,
                is_vertical: true,
            });
            self.walls.push(MazeWall {
                x1: 46,
                y1: 26,
                x2: 82,
                y2: 26,
                is_vertical: false,
            });
            self.walls.push(MazeWall {
                x1: 46,
                y1: 48,
                x2: 82,
                y2: 48,
                is_vertical: false,
            });
            self.walls.push(MazeWall {
                x1: 64,
                y1: 34,
                x2: 64,
                y2: 48,
                is_vertical: true,
            });
        }

        // 3. 随机置入微观反弹小障碍立柱
        if self.rng.random_range(0..100) < 65 {
            self.walls.push(MazeWall {
                x1: 64,
                y1: 15,
                x2: 64,
                y2: 21,
                is_vertical: true,
            });
        }
        if self.rng.random_range(0..100) < 65 {
            self.walls.push(MazeWall {
                x1: 52,
                y1: 37,
                x2: 58,
                y2: 37,
                is_vertical: false,
            });
        }
    }

    fn respawn_enemy(&mut self) {
        self.enemy.active = true;
        self.enemy.last_shot = Instant::now();
        self.enemy.turret_angle = 0.0;
        self.enemy.sweep_dir = 1.0;
        self.enemy.stuck_frames = 0;
        self.ai_detour_ticks = 0;

        self.enemy.x = if self.player_x > 64.0 { 11.0 } else { 116.0 };
        self.enemy.y = 20.0;
        self.enemy.last_x = self.enemy.x;
        self.enemy.last_y = self.enemy.y;

        self.enemy.target_x = self.player_x;
        self.enemy.target_y = self.player_y;
    }
}

fn check_line_of_sight(ax: f32, ay: f32, bx: f32, by: f32, walls: &[MazeWall]) -> bool {
    for wall in walls {
        if wall.is_vertical {
            if (ax < wall.x1 as f32 && bx > wall.x1 as f32)
                || (ax > wall.x1 as f32 && bx < wall.x1 as f32)
            {
                let t = (wall.x1 as f32 - ax) / (bx - ax);
                let intersect_y = ay + t * (by - ay);
                if intersect_y >= wall.y1 as f32 && intersect_y <= wall.y2 as f32 {
                    return false;
                }
            }
        } else {
            if (ay < wall.y1 as f32 && by > wall.y1 as f32)
                || (ay > wall.y1 as f32 && by < wall.y1 as f32)
            {
                let t = (wall.y1 as f32 - ay) / (by - ay);
                let intersect_x = ax + t * (bx - ax);
                if intersect_x >= wall.x1 as f32 && intersect_x <= wall.x2 as f32 {
                    return false;
                }
            }
        }
    }
    true
}

pub fn update(ctx: &mut UpdateContext, state: &mut TankTroubleState) -> Option<App> {
    if ctx.menu_events.contains(UiEvents::KEY_ESC) {
        return Some(App::games_menu());
    }

    if state.is_game_over {
        if ctx.menu_events.contains(UiEvents::KEY_7) || ctx.menu_events.contains(UiEvents::CONFIRM)
        {
            *state = TankTroubleState::default();
        }
        return None;
    }

    // 计算帧增量 Delta Time (标定基准为 25ms 每帧)
    let elapsed = state.last_tick.elapsed().as_millis();
    if elapsed == 0 {
        return None;
    }
    state.last_tick = Instant::now();
    let mut dt = (elapsed as f32) / 25.0;
    if dt > 3.0 {
        dt = 3.0;
    }

    let move_speed = 1.2 * dt;

    let mut next_px = state.player_x;
    if ctx.input_manager.is_down(UiEvents::LEFT) {
        next_px -= move_speed;
    } else if ctx.input_manager.is_down(UiEvents::RIGHT) {
        next_px += move_speed;
    }

    for wall in &state.walls {
        if wall.is_vertical
            && state.player_y >= (wall.y1 - 4) as f32
            && state.player_y <= (wall.y2 + 4) as f32
            && (next_px - wall.x1 as f32).abs() < 5.0
        {
            if state.player_x < wall.x1 as f32 {
                next_px = wall.x1 as f32 - 5.0;
            } else {
                next_px = wall.x1 as f32 + 5.0;
            }
        }
    }
    state.player_x = next_px;

    let mut next_py = state.player_y;
    if ctx.input_manager.is_down(UiEvents::UP) {
        next_py -= move_speed;
    } else if ctx.input_manager.is_down(UiEvents::DOWN) {
        next_py += move_speed;
    }

    for wall in &state.walls {
        if !wall.is_vertical
            && state.player_x >= (wall.x1 - 4) as f32
            && state.player_x <= (wall.x2 + 4) as f32
            && (next_py - wall.y1 as f32).abs() < 5.0
        {
            if state.player_y < wall.y1 as f32 {
                next_py = wall.y1 as f32 - 5.0;
            } else {
                next_py = wall.y1 as f32 + 5.0;
            }
        }
    }
    state.player_y = next_py;

    if ctx.input_manager.is_down(UiEvents::KEY_5) {
        state.player_angle -= 0.08 * dt;
    }
    if ctx.input_manager.is_down(UiEvents::KEY_6) {
        state.player_angle += 0.08 * dt;
    }

    if state.player_angle < 0.0 {
        state.player_angle += 2.0 * std::f32::consts::PI;
    } else if state.player_angle > 2.0 * std::f32::consts::PI {
        state.player_angle -= 2.0 * std::f32::consts::PI;
    }

    // 3. 玩家手动开火击发
    if ctx.menu_events.contains(UiEvents::KEY_7)
        && state.last_player_fire.elapsed().as_millis() > 400
    {
        state.last_player_fire = Instant::now();
        for bullet in &mut state.bullets {
            if !bullet.active {
                bullet.active = true;
                bullet.bounces_left = 5;
                bullet.from_enemy = false;
                bullet.x = state.player_x + state.player_angle.cos() * 6.0;
                bullet.y = state.player_y + state.player_angle.sin() * 6.0;
                bullet.vx = state.player_angle.cos() * 2.6;
                bullet.vy = state.player_angle.sin() * 2.6;
                break;
            }
        }
    }

    // 4. 子弹流体反弹与弹幕伤害链结算
    for i in 0..MAX_TRBL_BULLETS {
        if !state.bullets[i].active {
            continue;
        }

        let mut b_next_x = state.bullets[i].x + state.bullets[i].vx * dt;
        let mut b_next_y = state.bullets[i].y + state.bullets[i].vy * dt;

        for wall in &state.walls {
            if wall.is_vertical {
                if state.bullets[i].y >= wall.y1 as f32
                    && state.bullets[i].y <= wall.y2 as f32
                    && ((state.bullets[i].x <= wall.x1 as f32
                        && b_next_x >= wall.x1 as f32
                        && state.bullets[i].vx > 0.0)
                        || (state.bullets[i].x >= wall.x1 as f32
                            && b_next_x <= wall.x1 as f32
                            && state.bullets[i].vx < 0.0))
                {
                    state.bullets[i].vx = -state.bullets[i].vx;
                    b_next_x =
                        state.bullets[i].x + if state.bullets[i].vx > 0.0 { 1.0 } else { -1.0 };
                    state.bullets[i].bounces_left -= 1;
                }
            } else {
                if state.bullets[i].x >= wall.x1 as f32
                    && state.bullets[i].x <= wall.x2 as f32
                    && ((state.bullets[i].y <= wall.y1 as f32
                        && b_next_y >= wall.y1 as f32
                        && state.bullets[i].vy > 0.0)
                        || (state.bullets[i].y >= wall.y1 as f32
                            && b_next_y <= wall.y1 as f32
                            && state.bullets[i].vy < 0.0))
                {
                    state.bullets[i].vy = -state.bullets[i].vy;
                    b_next_y =
                        state.bullets[i].y + if state.bullets[i].vy > 0.0 { 1.0 } else { -1.0 };
                    state.bullets[i].bounces_left -= 1;
                }
            }
        }

        state.bullets[i].x = b_next_x;
        state.bullets[i].y = b_next_y;

        // 边界销毁
        if state.bullets[i].bounces_left <= 0
            || state.bullets[i].x < 2.0
            || state.bullets[i].x > 126.0
            || state.bullets[i].y < 11.0
            || state.bullets[i].y > 62.0
        {
            state.bullets[i].active = false;
            continue;
        }

        // 击中玩家判定 (排除刚射出时的自伤)
        if state.bullets[i].from_enemy || state.bullets[i].bounces_left < 5 {
            let d_to_player = (state.bullets[i].x - state.player_x).powi(2)
                + (state.bullets[i].y - state.player_y).powi(2);
            if d_to_player < 18.0 {
                state.bullets[i].active = false;
                state.is_game_over = true;
                return None;
            }
        }

        // 击中人机判定
        if state.enemy.active && (!state.bullets[i].from_enemy || state.bullets[i].bounces_left < 5)
        {
            let d_to_enemy = (state.bullets[i].x - state.enemy.x).powi(2)
                + (state.bullets[i].y - state.enemy.y).powi(2);
            if d_to_enemy < 18.0 {
                state.bullets[i].active = false;
                state.enemy.active = false;
                state.score += 1;
                state.respawn_enemy();
            }
        }
    }

    // 5. 高性能人机智能寻路与跟踪 AI 逻辑
    if state.enemy.active {
        let see_player = check_line_of_sight(
            state.enemy.x,
            state.enemy.y,
            state.player_x,
            state.player_y,
            &state.walls,
        );

        // 防卡死死锁判定
        if (state.enemy.x - state.enemy.last_x).abs() < 0.04
            && (state.enemy.y - state.enemy.last_y).abs() < 0.04
        {
            state.enemy.stuck_frames += 1;
            if state.enemy.stuck_frames > 12 {
                state.enemy.target_x = state.rng.random_range(20..110) as f32;
                state.enemy.target_y = state.rng.random_range(18..56) as f32;
                state.enemy.stuck_frames = 0;
                state.ai_detour_ticks = 35;
            }
        } else {
            state.enemy.stuck_frames = 0;
        }
        state.enemy.last_x = state.enemy.x;
        state.enemy.last_y = state.enemy.y;

        if !see_player {
            if state.ai_detour_ticks > 0 {
                state.ai_detour_ticks -= 1;
            } else {
                state.enemy.target_x = state.player_x;
                state.enemy.target_y = state.player_y;
            }
        }

        let target_x = if see_player {
            state.player_x
        } else {
            state.enemy.target_x
        };
        let target_y = if see_player {
            state.player_y
        } else {
            state.enemy.target_y
        };

        let edx = target_x - state.enemy.x;
        let edy = target_y - state.enemy.y;
        let e_dist = (edx * edx + edy * edy).sqrt();

        if e_dist < 4.0 && !see_player {
            state.ai_detour_ticks = 0;
            state.enemy.target_x = state.player_x;
            state.enemy.target_y = state.player_y;
        } else if e_dist > 1.0 {
            let enemy_speed = if see_player { 0.85 } else { 0.55 } * dt;
            let dir_x = edx / e_dist;
            let dir_y = edy / e_dist;

            let mut e_next_x = state.enemy.x + dir_x * enemy_speed;
            for wall in &state.walls {
                if wall.is_vertical
                    && state.enemy.y >= (wall.y1 - 4) as f32
                    && state.enemy.y <= (wall.y2 + 4) as f32
                    && (e_next_x - wall.x1 as f32).abs() < 5.0
                {
                    e_next_x = wall.x1 as f32
                        + if state.enemy.x < wall.x1 as f32 {
                            -5.0
                        } else {
                            5.0
                        };
                }
            }
            state.enemy.x = e_next_x;

            let mut e_next_y = state.enemy.y + dir_y * enemy_speed;
            for wall in &state.walls {
                if !wall.is_vertical
                    && state.enemy.x >= (wall.x1 - 4) as f32
                    && state.enemy.x <= (wall.x2 + 4) as f32
                    && (e_next_y - wall.y1 as f32).abs() < 5.0
                {
                    e_next_y = wall.y1 as f32
                        + if state.enemy.y < wall.y1 as f32 {
                            -5.0
                        } else {
                            5.0
                        };
                }
            }
            state.enemy.y = e_next_y;
        }

        // AI 锁敌及开火逻辑
        if see_player {
            let target_angle =
                (state.player_y - state.enemy.y).atan2(state.player_x - state.enemy.x);
            let mut angle_diff = target_angle - state.enemy.turret_angle;

            while angle_diff < -std::f32::consts::PI {
                angle_diff += 2.0 * std::f32::consts::PI;
            }
            while angle_diff > std::f32::consts::PI {
                angle_diff -= 2.0 * std::f32::consts::PI;
            }

            let max_rotation = 0.07 * dt;
            if angle_diff.abs() <= max_rotation {
                state.enemy.turret_angle = target_angle;
            } else {
                state.enemy.turret_angle += if angle_diff > 0.0 {
                    max_rotation
                } else {
                    -max_rotation
                };
            }

            let mut current_diff = target_angle - state.enemy.turret_angle;
            while current_diff < -std::f32::consts::PI {
                current_diff += 2.0 * std::f32::consts::PI;
            }
            while current_diff > std::f32::consts::PI {
                current_diff -= 2.0 * std::f32::consts::PI;
            }

            let check_dist = 8.0;
            let front_x = state.enemy.x + state.enemy.turret_angle.cos() * check_dist;
            let front_y = state.enemy.y + state.enemy.turret_angle.sin() * check_dist;
            let clear_front =
                check_line_of_sight(state.enemy.x, state.enemy.y, front_x, front_y, &state.walls);

            if clear_front
                && current_diff.abs() < 0.21
                && state.enemy.last_shot.elapsed().as_millis() > 800
            {
                state.enemy.last_shot = Instant::now();
                for bullet in &mut state.bullets {
                    if !bullet.active {
                        bullet.active = true;
                        bullet.bounces_left = 5;
                        bullet.from_enemy = true;
                        bullet.x = state.enemy.x + state.enemy.turret_angle.cos() * 5.0;
                        bullet.y = state.enemy.y + state.enemy.turret_angle.sin() * 5.0;
                        bullet.vx = state.enemy.turret_angle.cos() * 2.5;
                        bullet.vy = state.enemy.turret_angle.sin() * 2.5;
                        break;
                    }
                }
            }
        } else {
            // 丢失视野时维持广域扫射雷达表现
            state.enemy.turret_angle += 0.08 * state.enemy.sweep_dir * dt;
            let center_angle =
                (state.player_y - state.enemy.y).atan2(state.player_x - state.enemy.x);
            if (state.enemy.turret_angle - center_angle).abs() > 0.6 {
                state.enemy.sweep_dir = -state.enemy.sweep_dir;
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext<'_, '_>, state: &TankTroubleState) {
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

    let mut score_bytes = [0u8; 16];
    let score_str = msg_format(&mut score_bytes, "TANK WIN: ", state.score);
    ui.label(rect_score, score_str).center().draw();
    ui.horizontal_divider(rect_divider);

    for wall in &state.walls {
        ui.draw_stroke_rect(
            Rect::new(
                wall.x1 as i32,
                wall.y1 as i32,
                (wall.x2 - wall.x1).unsigned_abs() as u32 + 1,
                (wall.y2 - wall.y1).unsigned_abs() as u32 + 1,
            ),
            BinaryColor::On,
            1,
        );
    }

    // player tank
    let px = state.player_x as i32;
    let py = state.player_y as i32;
    ui.draw_stroke_rect(Rect::new(px - 3, py - 3, 7, 7), BinaryColor::On, 1);
    ui.draw_filled_rect(Rect::new(px, py, 1, 1), BinaryColor::On);
    // gun
    let gun_x = px + (state.player_angle.cos() * 6.0) as i32;
    let gun_y = py + (state.player_angle.sin() * 6.0) as i32;
    ui.draw_stroke_rect(Rect::new(gun_x, gun_y, 1, 1), BinaryColor::On, 1);

    // ai tank
    if state.enemy.active {
        let ex = state.enemy.x as i32;
        let ey = state.enemy.y as i32;
        ui.draw_stroke_rect(Rect::new(ex - 3, ey - 3, 7, 7), BinaryColor::On, 1);
        ui.draw_filled_rect(Rect::new(ex - 1, ey - 1, 3, 3), BinaryColor::Off); // 挖空核心做出区别
        // gun
        let e_gun_x = ex + (state.enemy.turret_angle.cos() * 6.0) as i32;
        let e_gun_y = ey + (state.enemy.turret_angle.sin() * 6.0) as i32;
        ui.draw_stroke_rect(Rect::new(e_gun_x, e_gun_y, 1, 1), BinaryColor::On, 1);
    }

    // draw bullets
    for bullet in &state.bullets {
        if bullet.active {
            ui.draw_filled_rect(
                Rect::new(bullet.x as i32, bullet.y as i32, 1, 1),
                BinaryColor::On,
            );
        }
    }

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

        ui.label(rect_line1, "YOU DIED!").center().draw();
        ui.label(rect_line2, "[7] Replay").center().draw();
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
