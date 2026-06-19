use crate::{
    display::{Sh1106Unified, Ssd1306Unified},
    input::InputManager,
    rtc::ds1302::Ds1302,
    ui::ctx::UiEvents,
};

pub struct AppContext<'a, 'b> {
    /// 0.96'' display handle
    pub display_0_96: Ssd1306Unified<'a, 'b>,
    /// 1.3'' display handle
    pub display_1_3: Sh1106Unified<'a, 'b>,

    /// Font renderer
    pub font: &'a u8g2_fonts::FontRenderer,
    pub font_large: &'a u8g2_fonts::FontRenderer,

    pub input: &'a InputManager<'static>,

    pub uptime_secs: u64,
}

pub struct UpdateContext<'a, 'b> {
    pub menu_events: UiEvents,
    pub rtc: &'a mut Ds1302<'b>,
    pub input_manager: &'a InputManager<'static>,
}
