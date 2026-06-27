use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::UnifiedDisplay,
    ui::{
        Rect, Ui,
        ctx::UiEvents,
        layout::{FlexDirection, FlexNode},
    },
};
use u8g2_fonts::types::{HorizontalAlignment, VerticalPosition};

/// Interactive button mapping matrix
const GRID: [[&str; 4]; 5] = [
    ["C", "(", ")", "/"],
    ["7", "8", "9", "*"],
    ["4", "5", "6", "-"],
    ["1", "2", "3", "+"],
    ["0", ".", "Del", "="],
];

pub struct CalculatorState {
    pub tick: u32,
    pub expr: String,
    pub result: String,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            tick: 0,
            expr: String::new(),
            result: String::from("0"),
            cursor_row: 1, // Start focused on number 7
            cursor_col: 0,
        }
    }
}

// --- Expression Parser Engine (Shunting-Yard Method) ---

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Op(char),
    LParen,
    RParen,
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '+' | '-' | '*' | '/' => {
                tokens.push(Token::Op(c));
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_ascii_digit() || nc == '.' {
                        num_str.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let num = num_str
                    .parse::<f64>()
                    .map_err(|_| "Num Error".to_string())?;
                tokens.push(Token::Num(num));
            }
            _ => return Err(format!("Bad Token: {}", c)),
        }
    }
    Ok(tokens)
}

fn precedence(op: char) -> i32 {
    match op {
        '+' | '-' => 1,
        '*' | '/' => 2,
        _ => 0,
    }
}

fn apply_op(ops: &mut Vec<char>, vals: &mut Vec<f64>) -> Result<(), String> {
    let op = ops.pop().ok_or("Op error")?;
    let val2 = vals.pop().ok_or("Syntax error")?;
    let val1 = vals.pop().ok_or("Syntax error")?;
    let res = match op {
        '+' => val1 + val2,
        '-' => val1 - val2,
        '*' => val1 * val2,
        '/' => {
            if val2 == 0.0 {
                return Err("Div by 0".to_string());
            }
            val1 / val2
        }
        _ => return Err("Invalid Op".to_string()),
    };
    vals.push(res);
    Ok(())
}

pub fn evaluate(expr: &str) -> Result<f64, String> {
    let tokens = tokenize(expr)?;
    let mut values = Vec::new();
    let mut ops = Vec::new();

    let mut can_be_unary = true;
    let mut iter = tokens.into_iter().peekable();

    while let Some(token) = iter.next() {
        match token {
            Token::Num(n) => {
                values.push(n);
                can_be_unary = false;
            }
            Token::LParen => {
                ops.push('(');
                can_be_unary = true;
            }
            Token::RParen => {
                while let Some(&top) = ops.last() {
                    if top == '(' {
                        break;
                    }
                    apply_op(&mut ops, &mut values)?;
                }
                if ops.pop() != Some('(') {
                    return Err("Mismatched ()".to_string());
                }
                can_be_unary = false;
            }
            Token::Op(c) => {
                if c == '-' && can_be_unary {
                    if let Some(Token::Num(n)) = iter.next() {
                        values.push(-n);
                        can_be_unary = false;
                    } else {
                        return Err("Bad Negative".to_string());
                    }
                } else {
                    while let Some(&top) = ops.last() {
                        if top != '(' && precedence(top) >= precedence(c) {
                            apply_op(&mut ops, &mut values)?;
                        } else {
                            break;
                        }
                    }
                    ops.push(c);
                    can_be_unary = true;
                }
            }
        }
    }

    while !ops.is_empty() {
        if *ops.last().unwrap() == '(' {
            return Err("Mismatched ()".to_string());
        }
        apply_op(&mut ops, &mut values)?;
    }

    if values.len() == 1 {
        Ok(values[0])
    } else if values.is_empty() {
        Err("Empty Expr".to_string())
    } else {
        Err("Syntax Error".to_string())
    }
}

pub fn update(ctx: &UpdateContext, state: &mut CalculatorState) -> Option<App> {
    state.tick += 1;
    let events = ctx.menu_events;

    if events.intersects(UiEvents::KEY_ESC) {
        return Some(App::main_menu());
    }

    // Grid Navigation Processing
    if events.contains(UiEvents::UP) {
        state.cursor_row = state.cursor_row.saturating_sub(1);
    }
    if events.contains(UiEvents::DOWN) && state.cursor_row < 4 {
        state.cursor_row += 1;
    }
    if events.contains(UiEvents::LEFT) {
        state.cursor_col = state.cursor_col.saturating_sub(1);
    }
    if events.contains(UiEvents::RIGHT) && state.cursor_col < 3 {
        state.cursor_col += 1;
    }

    // Selection Submission Handling
    if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) {
        let cmd = GRID[state.cursor_row][state.cursor_col];
        match cmd {
            "C" => {
                state.expr.clear();
                state.result = String::from("0");
            }
            "Del" => {
                state.expr.pop();
            }
            "=" => {
                if !state.expr.is_empty() {
                    match evaluate(&state.expr) {
                        Ok(val) => {
                            if val.fract() == 0.0 {
                                state.result = format!("{}", val as i64);
                            } else {
                                state.result = format!("{:.4}", val);
                            }
                        }
                        Err(err) => {
                            state.result = err;
                        }
                    }
                }
            }
            _ => {
                state.expr.push_str(cmd);
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &CalculatorState) {
    // 1. Auxiliary Sub Display Handling (0.96")
    let sub_bounds = ctx.display_0_96.rect();
    let mut sub_ui = Ui::new(&mut ctx.display_0_96, ctx.font);

    let sub_title_rect = Rect::new(sub_bounds.x, sub_bounds.y, sub_bounds.width, 12);
    let sub_content_rect = Rect::new(
        sub_bounds.x,
        sub_bounds.y + 12,
        sub_bounds.width,
        sub_bounds.height.saturating_sub(12),
    );

    sub_ui.label(sub_title_rect, "CALC MODE").center().draw();
    if state.expr.is_empty() {
        sub_ui.label(sub_content_rect, "Ready").center().draw();
    } else {
        sub_ui
            .label(sub_content_rect, &state.expr)
            .scroll(state.tick, 1)
            .draw();
    }

    // 2. Primary Layout Framework Setup (1.3")
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut header_rect = Rect::default();
    let mut top_divider_rect = Rect::default();
    let mut display_area_rect = Rect::default();
    let mut bottom_divider_rect = Rect::default();
    let mut grid_container_rect = Rect::default();

    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 13)
                .assign_to(&mut header_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 2)
                .assign_to(&mut top_divider_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 22)
                .assign_to(&mut display_area_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 1)
                .assign_to(&mut bottom_divider_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut grid_container_rect),
        )
        .layout(display_bounds);

    ui.label(header_rect, "CALCULATOR").center().draw();
    ui.horizontal_divider(top_divider_rect);

    // Render Calculator Internal Screen Content Bounds
    let expr_rect = Rect::new(
        display_area_rect.x + 2,
        display_area_rect.y,
        display_area_rect.width - 4,
        10,
    );
    let result_rect = Rect::new(
        display_area_rect.x + 2,
        display_area_rect.y + 11,
        display_area_rect.width - 4,
        11,
    );

    let display_expr = if state.expr.is_empty() {
        "_"
    } else {
        &state.expr
    };
    ui.label(expr_rect, display_expr)
        .align(HorizontalAlignment::Right, VerticalPosition::Top)
        .draw();

    ui.label(result_rect, &state.result)
        .align(HorizontalAlignment::Right, VerticalPosition::Top)
        .draw();

    ui.horizontal_divider(bottom_divider_rect);

    // Uniformly partition and slice grid boundaries inside container
    let row_height = grid_container_rect.height / 5;
    let col_width = grid_container_rect.width / 4;

    for (r, row) in GRID.iter().enumerate().take(5) {
        for (c, &cell_value) in row.iter().enumerate().take(4) {
            let btn_x = grid_container_rect.x + (c as i32 * col_width as i32);
            let btn_y = grid_container_rect.y + (r as i32 * row_height as i32);

            let cell_rect = Rect::new(btn_x, btn_y, col_width, row_height);
            let is_selected = (r == state.cursor_row) && (c == state.cursor_col);

            ui.button(cell_rect, cell_value, is_selected);
        }
    }
}
