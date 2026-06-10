use esp_idf_svc::hal::gpio::{AnyIOPin, AnyInputPin, AnyOutputPin};

pub struct PinConfig {
    pub keyboard: KeyboardPinConfig,
    pub i2c_display: I2cDisplayPinConfig,
    pub rtc: RtcPinConfig,
    pub joy: JoyPinConfig,
}

pub struct KeyboardPinConfig {
    pub h: AnyInputPin<'static>,
    pub j: AnyInputPin<'static>,
    pub k: AnyInputPin<'static>,
    pub l: AnyInputPin<'static>,
    pub confirm: AnyInputPin<'static>,
    pub back: AnyInputPin<'static>,
}

pub struct I2cDisplayPinConfig {
    pub sda: AnyIOPin<'static>,
    pub scl: AnyIOPin<'static>,
    pub addr_1_3_in: u8,
    pub addr_0_96_in: u8,
}

/// DS1302 config
pub struct RtcPinConfig {
    pub clk: AnyOutputPin<'static>,
    pub dat: AnyIOPin<'static>,
    pub rst: AnyOutputPin<'static>,
}

pub struct JoyPinConfig {
    pub x: AnyInputPin<'static>,
    pub y: AnyInputPin<'static>,
    pub sw: AnyInputPin<'static>,
}
