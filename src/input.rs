use esp_idf_svc::hal::gpio::{AnyInputPin, Input, PinDriver, Pull};

use crate::ui::ctx::UiEvents;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoystickRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct JoystickData {
    pub x: f32,
    pub y: f32,
    pub is_pressed: bool,
}

pub struct InputManager<'a> {
    pub btn_h: PinDriver<'a, Input>,
    pub btn_j: PinDriver<'a, Input>,
    pub btn_k: PinDriver<'a, Input>,
    pub btn_l: PinDriver<'a, Input>,
    pub btn_confirm: PinDriver<'a, Input>,
    pub btn_back: PinDriver<'a, Input>,
    pub btn_joy_sw: PinDriver<'a, Input>,

    pub joy_x_pin: AnyInputPin<'a>,
    pub joy_y_pin: AnyInputPin<'a>,
    pub joy_rotation: JoystickRotation,
}

impl<'a> InputManager<'a> {
    pub fn new(
        keyboard: crate::pin_config::KeyboardPinConfig,
        joy: crate::pin_config::JoyPinConfig,
        rotation: JoystickRotation,
    ) -> Result<Self, anyhow::Error> {
        let btn_h = PinDriver::input(keyboard.h, Pull::Up)?;
        let btn_j = PinDriver::input(keyboard.j, Pull::Up)?;
        let btn_k = PinDriver::input(keyboard.k, Pull::Up)?;
        let btn_l = PinDriver::input(keyboard.l, Pull::Up)?;
        let btn_confirm = PinDriver::input(keyboard.confirm, Pull::Up)?;
        let btn_back = PinDriver::input(keyboard.back, Pull::Up)?;
        let btn_joy_sw = PinDriver::input(joy.sw, Pull::Up)?;

        unsafe {
            esp_idf_svc::sys::adc1_config_width(
                esp_idf_svc::sys::adc_bits_width_t_ADC_WIDTH_BIT_12,
            );

            esp_idf_svc::sys::adc1_config_channel_atten(
                esp_idf_svc::sys::adc1_channel_t_ADC1_CHANNEL_2,
                esp_idf_svc::sys::adc_atten_t_ADC_ATTEN_DB_11,
            );
            esp_idf_svc::sys::adc1_config_channel_atten(
                esp_idf_svc::sys::adc1_channel_t_ADC1_CHANNEL_6,
                esp_idf_svc::sys::adc_atten_t_ADC_ATTEN_DB_11,
            );
        }

        Ok(Self {
            btn_h,
            btn_j,
            btn_k,
            btn_l,
            btn_confirm,
            btn_back,
            btn_joy_sw,
            joy_x_pin: joy.x,
            joy_y_pin: joy.y,
            joy_rotation: rotation,
        })
    }

    pub fn read_joystick(&self) -> JoystickData {
        let raw_x = unsafe {
            esp_idf_svc::sys::adc1_get_raw(esp_idf_svc::sys::adc1_channel_t_ADC1_CHANNEL_2)
        };
        let raw_y = unsafe {
            esp_idf_svc::sys::adc1_get_raw(esp_idf_svc::sys::adc1_channel_t_ADC1_CHANNEL_6)
        };

        let x_norm = (raw_x as f32 - 2048.0) / 2048.0;
        let y_norm = (raw_y as f32 - 2048.0) / 2048.0;

        let x_clamped = x_norm.clamp(-1.0, 1.0);
        let y_clamped = y_norm.clamp(-1.0, 1.0);

        let (rx, ry) = match self.joy_rotation {
            JoystickRotation::Deg0 => (x_clamped, y_clamped),
            JoystickRotation::Deg90 => (-y_clamped, x_clamped),
            JoystickRotation::Deg180 => (-x_clamped, -y_clamped),
            JoystickRotation::Deg270 => (y_clamped, -x_clamped),
        };

        JoystickData {
            x: rx,
            y: ry,
            is_pressed: self.btn_joy_sw.is_low(),
        }
    }

    pub fn get_ui_events(&self, joy_data: JoystickData) -> UiEvents {
        let mut events = UiEvents::empty();

        if self.btn_confirm.is_low() {
            events.insert(UiEvents::PRIMARY_CONFIRM);
        }
        if self.btn_back.is_low() {
            events.insert(UiEvents::BACK);
        }
        if self.btn_k.is_low() {
            events.insert(UiEvents::PRIMARY_UP);
        }
        if self.btn_j.is_low() {
            events.insert(UiEvents::PRIMARY_DOWN);
        }
        if self.btn_h.is_low() {
            events.insert(UiEvents::PRIMARY_LEFT);
        }
        if self.btn_l.is_low() {
            events.insert(UiEvents::PRIMARY_RIGHT);
        }

        if joy_data.y > 0.5 {
            events.insert(UiEvents::SECONDARY_UP);
        } else if joy_data.y < -0.5 {
            events.insert(UiEvents::SECONDARY_DOWN);
        }

        if joy_data.x < -0.5 {
            events.insert(UiEvents::SECONDARY_LEFT);
        } else if joy_data.x > 0.5 {
            events.insert(UiEvents::SECONDARY_RIGHT);
        }

        if joy_data.is_pressed {
            events.insert(UiEvents::SECONDARY_CONFIRM);
        }

        events
    }
}
