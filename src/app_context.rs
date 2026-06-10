use crate::{
    display::{Sh1106Unified, Ssd1306Unified},
    input::InputManager,
    rtc::ds1302::Ds1302,
    ui::UiEvent,
};

pub struct AppContext<'a, 'b> {
    /// 0.96'' display handle
    pub display_0_96: Ssd1306Unified<'a, 'b>,
    /// 1.3'' display handle
    pub display_1_3: Sh1106Unified<'a, 'b>,

    /// Font renderer
    pub font: &'a u8g2_fonts::FontRenderer,
    pub font_large: &'a u8g2_fonts::FontRenderer,

    pub input: &'a mut InputManager<'static>,

    pub uptime_secs: u64,
    pub current_event: UiEvent,
}

pub struct UpdateContext<'a, 'b> {
    pub event: UiEvent,
    pub rtc: &'a mut Ds1302<'b>,
}
