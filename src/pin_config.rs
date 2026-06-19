use esp_idf_svc::hal::gpio::{AnyIOPin, AnyInputPin, AnyOutputPin};

pub struct PinConfig {
    pub keyboard: KeyboardMatrixConfig,
    pub i2c_display: I2cDisplayPinConfig,
    pub rtc: RtcPinConfig,
    pub joy: JoyPinConfig,
}

pub struct KeyboardMatrixConfig {
    pub rows: [AnyOutputPin<'static>; 2],
    pub cols: [AnyInputPin<'static>; 4],
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
