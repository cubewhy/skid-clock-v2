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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcMode {
    Scientific,
    Programmer,
    Converter,
}

impl CalcMode {
    pub fn name(&self) -> &'static str {
        match self {
            CalcMode::Scientific => "SCIENTIFIC",
            CalcMode::Programmer => "PROGRAMMER",
            CalcMode::Converter => "CONVERTER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumBase {
    Dec,
    Hex,
    Bin,
    Oct,
}

impl NumBase {
    pub fn name(&self) -> &'static str {
        match self {
            NumBase::Dec => "DEC",
            NumBase::Hex => "HEX",
            NumBase::Bin => "BIN",
            NumBase::Oct => "OCT",
        }
    }
}

pub struct CalculatorState {
    pub tick: u32,
    pub expr: String,
    pub result: String,
    pub current_mode: CalcMode,
    pub programmer_base: NumBase,
    pub current_layer: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            tick: 0,
            expr: String::new(),
            result: String::from("0"),
            current_mode: CalcMode::Scientific,
            programmer_base: NumBase::Dec,
            current_layer: 0,
            cursor_row: 0,
            cursor_col: 0,
        }
    }
}

/// Dynamic Key Matrix Provider based on Modes and Layers
fn get_grid(mode: CalcMode, layer: usize) -> [[&'static str; 4]; 5] {
    match mode {
        CalcMode::Scientific => {
            if layer == 0 {
                [
                    ["C", "(", ")", "/"],
                    ["7", "8", "9", "*"],
                    ["4", "5", "6", "-"],
                    ["1", "2", "3", "+"],
                    ["0", ".", "Del", "="],
                ]
            } else {
                [
                    ["sin", "cos", "tan", "^"],
                    ["sqrt", "ln", "log", "PI"],
                    ["E", "abs", "(", ")"],
                    [" ", " ", " ", " "],
                    ["C", " ", "Del", "="],
                ]
            }
        }
        CalcMode::Programmer => {
            if layer == 0 {
                [
                    ["C", "(", ")", "/"],
                    ["7", "8", "9", "*"],
                    ["4", "5", "6", "-"],
                    ["1", "2", "3", "+"],
                    ["0", "Del", "BASE", "="],
                ]
            } else {
                [
                    ["A", "B", "C", "D"],
                    ["E", "F", "&", "|"],
                    ["XOR", "~", "<<", ">>"],
                    [" ", " ", " ", " "],
                    ["C", "Del", "BASE", "="],
                ]
            }
        }
        CalcMode::Converter => {
            if layer == 0 {
                [
                    ["C", " ", " ", " "],
                    ["7", "8", "9", "C->F"],
                    ["4", "5", "6", "F->C"],
                    ["1", "2", "3", "M->Ft"],
                    ["0", ".", "Del", "Ft->M"],
                ]
            } else {
                [
                    ["C", " ", " ", " "],
                    ["7", "8", "9", "Kg->Lb"],
                    ["4", "5", "6", "Lb->Kg"],
                    ["1", "2", "3", "L->Gal"],
                    ["0", ".", "Del", "Gal->L"],
                ]
            }
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq)]
enum OpType {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
    LParen,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Op(OpType),
    Func(&'static str),
    LParen,
    RParen,
}

fn tokenize(expr: &str, current_base: NumBase) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '+' => {
                tokens.push(Token::Op(OpType::Add));
                chars.next();
            }
            '-' => {
                tokens.push(Token::Op(OpType::Sub));
                chars.next();
            }
            '*' => {
                tokens.push(Token::Op(OpType::Mul));
                chars.next();
            }
            '/' => {
                tokens.push(Token::Op(OpType::Div));
                chars.next();
            }
            '^' => {
                // In Programmer mode context, treating '^' as XOR, otherwise Power
                if current_base != NumBase::Dec {
                    tokens.push(Token::Op(OpType::Xor));
                } else {
                    tokens.push(Token::Op(OpType::Pow));
                }
                chars.next();
            }
            '&' => {
                tokens.push(Token::Op(OpType::And));
                chars.next();
            }
            '|' => {
                tokens.push(Token::Op(OpType::Or));
                chars.next();
            }
            '~' => {
                tokens.push(Token::Op(OpType::Not));
                chars.next();
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'<') {
                    tokens.push(Token::Op(OpType::Shl));
                    chars.next();
                } else {
                    return Err("Bad Shl Token".to_string());
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    tokens.push(Token::Op(OpType::Shr));
                    chars.next();
                } else {
                    return Err("Bad Shr Token".to_string());
                }
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            _ => {
                if c.is_ascii_alphanumeric() || c == '.' {
                    let mut buffer = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc.is_ascii_alphanumeric() || nc == '.' {
                            buffer.push(nc);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    // 1. Check for explicit system base prefixes
                    if buffer.starts_with("0x") || buffer.starts_with("0X") {
                        let val = i64::from_str_radix(&buffer[2..], 16)
                            .map_err(|_| "Hex Parse Error".to_string())?;
                        tokens.push(Token::Num(val as f64));
                    } else if buffer.starts_with("0b") || buffer.starts_with("0B") {
                        let val = i64::from_str_radix(&buffer[2..], 2)
                            .map_err(|_| "Bin Parse Error".to_string())?;
                        tokens.push(Token::Num(val as f64));
                    } else if buffer.starts_with("0o") || buffer.starts_with("0O") {
                        let val = i64::from_str_radix(&buffer[2..], 8)
                            .map_err(|_| "Oct Parse Error".to_string())?;
                        tokens.push(Token::Num(val as f64));
                    }
                    // 2. Fallback to processing functions or constants
                    else {
                        match buffer.as_str() {
                            "sin" => tokens.push(Token::Func("sin")),
                            "cos" => tokens.push(Token::Func("cos")),
                            "tan" => tokens.push(Token::Func("tan")),
                            "sqrt" => tokens.push(Token::Func("sqrt")),
                            "ln" => tokens.push(Token::Func("ln")),
                            "log" => tokens.push(Token::Func("log")),
                            "abs" => tokens.push(Token::Func("abs")),
                            "PI" => tokens.push(Token::Num(std::f64::consts::PI)),
                            "E" if current_base == NumBase::Dec => {
                                tokens.push(Token::Num(std::f64::consts::E))
                            }
                            // 3. Contextual evaluation using current active base
                            _ => match current_base {
                                NumBase::Hex => {
                                    let val = i64::from_str_radix(&buffer, 16)
                                        .map_err(|_| format!("Invalid Hex: {}", buffer))?;
                                    tokens.push(Token::Num(val as f64));
                                }
                                NumBase::Bin => {
                                    let val = i64::from_str_radix(&buffer, 2)
                                        .map_err(|_| format!("Invalid Bin: {}", buffer))?;
                                    tokens.push(Token::Num(val as f64));
                                }
                                NumBase::Oct => {
                                    let val = i64::from_str_radix(&buffer, 8)
                                        .map_err(|_| format!("Invalid Oct: {}", buffer))?;
                                    tokens.push(Token::Num(val as f64));
                                }
                                NumBase::Dec => {
                                    let val = buffer
                                        .parse::<f64>()
                                        .map_err(|_| format!("Invalid Dec: {}", buffer))?;
                                    tokens.push(Token::Num(val));
                                }
                            },
                        }
                    }
                } else {
                    return Err(format!("Bad Character: {}", c));
                }
            }
        }
    }
    Ok(tokens)
}

fn precedence(op: &OpType) -> i32 {
    match op {
        OpType::And | OpType::Or | OpType::Xor | OpType::Not | OpType::Shl | OpType::Shr => 1,
        OpType::Add | OpType::Sub => 2,
        OpType::Mul | OpType::Div => 3,
        OpType::Pow => 4,
        OpType::LParen => 0,
    }
}

fn apply_op(
    ops: &mut Vec<OpType>,
    funcs: &mut Vec<&'static str>,
    vals: &mut Vec<f64>,
) -> Result<(), String> {
    if !funcs.is_empty() {
        let func = funcs.pop().unwrap();
        let val = vals.pop().ok_or("Missing arg")?;
        let res = match func {
            "sin" => val.sin(),
            "cos" => val.cos(),
            "tan" => val.tan(),
            "sqrt" => val.sqrt(),
            "ln" => val.ln(),
            "log" => val.log10(),
            "abs" => val.abs(),
            _ => return Err("Bad function lookup".to_string()),
        };
        vals.push(res);
        return Ok(());
    }

    let op = ops.pop().ok_or("Missing operator")?;

    if let OpType::Not = op {
        let val = vals.pop().ok_or("Syntax error")?;
        vals.push((!(val as i64)) as f64);
        return Ok(());
    }

    let val2 = vals.pop().ok_or("Syntax error")?;
    let val1 = vals.pop().ok_or("Syntax error")?;
    let res = match op {
        OpType::Add => val1 + val2,
        OpType::Sub => val1 - val2,
        OpType::Mul => val1 * val2,
        OpType::Div => {
            if val2 == 0.0 {
                return Err("Div by 0".to_string());
            }
            val1 / val2
        }
        OpType::And => (val1 as i64 & val2 as i64) as f64,
        OpType::Or => (val1 as i64 | val2 as i64) as f64,
        OpType::Xor => (val1 as i64 ^ val2 as i64) as f64,
        OpType::Shl => ((val1 as i64) << (val2 as i64)) as f64,
        OpType::Shr => ((val1 as i64) >> (val2 as i64)) as f64,
        OpType::Pow => val1.powf(val2),
        OpType::LParen => return Err("Mismatched Parenthesis".to_string()),
        OpType::Not => unreachable!(),
    };
    vals.push(res);
    Ok(())
}

pub fn evaluate(expr: &str, current_base: NumBase) -> Result<f64, String> {
    let tokens = tokenize(expr, current_base)?;
    let mut values = Vec::new();
    let mut ops = Vec::new();
    let mut funcs = Vec::new();

    for token in tokens {
        match token {
            Token::Num(n) => values.push(n),
            Token::Func(f) => funcs.push(f),
            Token::LParen => ops.push(OpType::LParen),
            Token::RParen => {
                while let Some(&top) = ops.last() {
                    if top == OpType::LParen {
                        break;
                    }
                    apply_op(&mut ops, &mut funcs, &mut values)?;
                }
                ops.pop(); // Remove LParen
                if !funcs.is_empty() {
                    apply_op(&mut ops, &mut funcs, &mut values)?;
                }
            }
            Token::Op(c) => {
                while let Some(&top) = ops.last() {
                    if top != OpType::LParen && precedence(&top) >= precedence(&c) {
                        apply_op(&mut ops, &mut funcs, &mut values)?;
                    } else {
                        break;
                    }
                }
                ops.push(c);
            }
        }
    }

    while !ops.is_empty() || !funcs.is_empty() {
        apply_op(&mut ops, &mut funcs, &mut values)?;
    }

    values.pop().ok_or_else(|| "Empty Result".to_string())
}

pub fn update(ctx: &UpdateContext, state: &mut CalculatorState) -> Option<App> {
    state.tick += 1;
    let events = ctx.menu_events;

    if events.intersects(UiEvents::KEY_ESC) {
        return Some(App::main_menu());
    }

    // Mode Rotation Toggle Engine [KEY_1]
    if events.intersects(UiEvents::KEY_1) {
        state.current_mode = match state.current_mode {
            CalcMode::Scientific => CalcMode::Programmer,
            CalcMode::Programmer => CalcMode::Converter,
            CalcMode::Converter => CalcMode::Scientific,
        };
        state.current_layer = 0;
        state.expr.clear();
        state.result = String::from("0");
    }

    // Active Matrix Character Layer Switch [KEY_2]
    if events.intersects(UiEvents::KEY_2) {
        state.current_layer = if state.current_layer == 0 { 1 } else { 0 };
    }

    // HJKL / Analog Joystick Matrix Grid Traversal Navigation Mapping
    if events.intersects(UiEvents::KEY_6 | UiEvents::UP) {
        state.cursor_row = state.cursor_row.saturating_sub(1);
    }
    if events.intersects(UiEvents::KEY_5 | UiEvents::DOWN) && state.cursor_row < 4 {
        state.cursor_row += 1;
    }
    if events.intersects(UiEvents::KEY_4 | UiEvents::LEFT) {
        state.cursor_col = state.cursor_col.saturating_sub(1);
    }
    if events.intersects(UiEvents::KEY_7 | UiEvents::RIGHT) && state.cursor_col < 3 {
        state.cursor_col += 1;
    }

    // Action Input Confirmation Selector [KEY_3 or CONFIRM]
    if events.intersects(UiEvents::KEY_3 | UiEvents::CONFIRM) {
        let grid = get_grid(state.current_mode, state.current_layer);
        let cmd = grid[state.cursor_row][state.cursor_col];

        if cmd != " " {
            match cmd {
                "C" => {
                    state.expr.clear();
                    state.result = String::from("0");
                }
                "Del" => {
                    state.expr.pop();
                }
                "BASE" => {
                    state.programmer_base = match state.programmer_base {
                        NumBase::Dec => NumBase::Hex,
                        NumBase::Hex => NumBase::Bin,
                        NumBase::Bin => NumBase::Oct,
                        NumBase::Oct => NumBase::Dec,
                    };
                }
                "=" => {
                    if !state.expr.is_empty() {
                        match evaluate(&state.expr, state.programmer_base) {
                            Ok(val) => {
                                if state.current_mode == CalcMode::Programmer && val.fract() == 0.0
                                {
                                    state.result = format!("{}", val as i64);
                                } else {
                                    state.result = format!("{:.4}", val);
                                }
                            }
                            Err(err) => state.result = err,
                        }
                    }
                }
                // --- Layer 0 Conversions ---
                "C->F" => {
                    if let Ok(val) = state.expr.parse::<f64>() {
                        state.result = format!("{:.2} F", (val * 9.0 / 5.0) + 32.0);
                    }
                }
                "F->C" => {
                    if let Ok(val) = state.expr.parse::<f64>() {
                        state.result = format!("{:.2} C", (val - 32.0) * 5.0 / 9.0);
                    }
                }
                "M->Ft" => {
                    if let Ok(val) = state.expr.parse::<f64>() {
                        state.result = format!("{:.2} Ft", val * 3.28084);
                    }
                }
                "Ft->M" => {
                    if let Ok(val) = state.expr.parse::<f64>() {
                        state.result = format!("{:.2} M", val / 3.28084);
                    }
                }
                // --- Layer 1 Conversions ---
                "Kg->Lb" => {
                    if let Ok(val) = state.expr.parse::<f64>() {
                        state.result = format!("{:.2} Lb", val * 2.20462);
                    }
                }
                "Lb->Kg" => {
                    if let Ok(val) = state.expr.parse::<f64>() {
                        state.result = format!("{:.2} Kg", val / 2.20462);
                    }
                }
                "L->Gal" => {
                    if let Ok(val) = state.expr.parse::<f64>() {
                        state.result = format!("{:.2} Gal", val * 0.264172);
                    }
                }
                "Gal->L" => {
                    if let Ok(val) = state.expr.parse::<f64>() {
                        state.result = format!("{:.2} L", val / 0.264172);
                    }
                }
                "sin" | "cos" | "tan" | "sqrt" | "ln" | "log" | "abs" => {
                    state.expr.push_str(cmd);
                    state.expr.push('(');
                }
                "XOR" => {
                    state.expr.push('^');
                }
                _ => {
                    state.expr.push_str(cmd);
                }
            }
        }
    }

    None
}

pub fn draw(ctx: &mut AppContext, state: &CalculatorState) {
    // ==========================================
    // 1. SUB SCREEN DRAW PATHWAY (0.96") -> KEYBOARD MATRIX GRID
    // ==========================================
    let sub_bounds = ctx.display_0_96.rect();
    let mut sub_ui = Ui::new(&mut ctx.display_0_96, ctx.font);

    sub_ui.draw_filled_rect(sub_bounds, embedded_graphics::pixelcolor::BinaryColor::Off);

    let grid = get_grid(state.current_mode, state.current_layer);
    let row_height = sub_bounds.height / 5;
    let col_width = sub_bounds.width / 4;

    for (r, row) in grid.iter().enumerate().take(5) {
        for (c, &cell_value) in row.iter().enumerate().take(4) {
            let btn_x = sub_bounds.x + (c as i32 * col_width as i32);
            let btn_y = sub_bounds.y + (r as i32 * row_height as i32);

            let cell_rect = Rect::new(
                btn_x + 1,
                btn_y + 1,
                col_width.saturating_sub(2),
                row_height.saturating_sub(2),
            );

            let is_selected = (r == state.cursor_row) && (c == state.cursor_col);
            sub_ui.button(cell_rect, cell_value, is_selected);
        }
    }

    // ==========================================
    // 2. MAIN SCREEN DRAW PATHWAY (1.3") -> STATUS & CALCULATION RESULTS
    // ==========================================
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    let mut header_rect = Rect::default();
    let mut top_divider_rect = Rect::default();
    let mut display_area_rect = Rect::default();
    let mut bottom_divider_rect = Rect::default();
    let mut guide_footer_rect = Rect::default();

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
                .with_flex(1)
                .assign_to(&mut display_area_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 2)
                .assign_to(&mut bottom_divider_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 12)
                .assign_to(&mut guide_footer_rect),
        )
        .layout(display_bounds);

    // Render Window Headers showing structural base context
    let header_text = if state.current_mode == CalcMode::Programmer {
        format!(
            "{} ({})",
            state.current_mode.name(),
            state.programmer_base.name()
        )
    } else {
        format!("CALC: {}", state.current_mode.name())
    };
    ui.label(header_rect, &header_text).center().draw();
    ui.horizontal_divider(top_divider_rect);

    // Render Calculation Display Fields Internally
    let expr_y = display_area_rect.y + 2;
    let expr_box = Rect::new(
        display_area_rect.x + 4,
        expr_y,
        display_area_rect.width - 8,
        11,
    );

    let display_expr = if state.expr.is_empty() {
        "_"
    } else {
        &state.expr
    };
    ui.label(expr_box, display_expr)
        .align(HorizontalAlignment::Right, VerticalPosition::Top)
        .draw();

    // Multi-Base Dynamic Presentation Logic for Programmer Viewports
    if state.current_mode == CalcMode::Programmer {
        let clean_num = state.result.parse::<i64>().unwrap_or(0);
        let layout_width = display_area_rect.width - 8;

        // Collect conversion formats except for the currently active base
        let mut dynamic_lines = Vec::new();
        if state.programmer_base != NumBase::Hex {
            dynamic_lines.push(format!("HEX: 0x{:X}", clean_num));
        }
        if state.programmer_base != NumBase::Dec {
            dynamic_lines.push(format!("DEC: {}", clean_num));
        }
        if state.programmer_base != NumBase::Oct {
            dynamic_lines.push(format!("OCT: 0o{:o}", clean_num));
        }
        if state.programmer_base != NumBase::Bin {
            dynamic_lines.push(format!("BIN: b{:b}", clean_num));
        }

        // Dynamically pack lines into the two rolling ticker rows
        let row1_str = format!("{}  {}", dynamic_lines[0], dynamic_lines[1]);
        ui.label(
            Rect::new(display_area_rect.x + 4, expr_y + 12, layout_width, 10),
            &row1_str,
        )
        .scroll(state.tick, 1)
        .draw();

        let row2_str = dynamic_lines[2].clone();
        ui.label(
            Rect::new(display_area_rect.x + 4, expr_y + 24, layout_width, 10),
            &row2_str,
        )
        .scroll(state.tick, 1)
        .draw();
    } else {
        // Standard high-capacity large output render maps
        let result_box = Rect::new(
            display_area_rect.x + 4,
            expr_y + 16,
            display_area_rect.width - 8,
            18,
        );
        ui.label(result_box, &state.result)
            .font(ctx.font_large)
            .align(HorizontalAlignment::Right, VerticalPosition::Top)
            .draw();
    }

    ui.horizontal_divider(bottom_divider_rect);

    // Bottom Navigation Help Banner Marquee Loop
    let guide_text = format!(
        "[L{}] [1]Mode [2]Layer [3]Enter [HJKL]Move [Esc]Exit",
        state.current_layer
    );
    ui.label(guide_footer_rect, &guide_text)
        .scroll(state.tick, 1)
        .draw();
}
