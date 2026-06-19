use crate::ui::ctx::UiEvents;
use esp_idf_svc::hal::gpio::{AnyInputPin, Input, Output, PinDriver, Pull};

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

impl JoystickData {
    pub fn get_events(&self) -> UiEvents {
        let mut events = UiEvents::empty();

        if self.y > 0.5 {
            events.insert(UiEvents::UP);
        } else if self.y < -0.5 {
            events.insert(UiEvents::DOWN);
        }

        if self.x < -0.5 {
            events.insert(UiEvents::LEFT);
        } else if self.x > 0.5 {
            events.insert(UiEvents::RIGHT);
        }

        if self.is_pressed {
            events.insert(UiEvents::CONFIRM);
        }

        events
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct KeyStates {
    pub esc: bool,
    pub btn_1: bool,
    pub btn_2: bool,
    pub btn_3: bool,
    pub btn_4: bool,
    pub btn_5: bool,
    pub btn_6: bool,
    pub btn_7: bool,
}

impl KeyStates {
    pub fn get_events(&self) -> UiEvents {
        let mut events = UiEvents::empty();

        if self.esc {
            events.insert(UiEvents::KEY_ESC);
        }
        if self.btn_1 {
            events.insert(UiEvents::KEY_1);
        }
        if self.btn_2 {
            events.insert(UiEvents::KEY_2);
        }
        if self.btn_3 {
            events.insert(UiEvents::KEY_3);
        }
        if self.btn_4 {
            events.insert(UiEvents::KEY_4);
        }
        if self.btn_5 {
            events.insert(UiEvents::KEY_5);
        }
        if self.btn_6 {
            events.insert(UiEvents::KEY_6);
        }
        if self.btn_7 {
            events.insert(UiEvents::KEY_7);
        }

        events
    }
}

pub struct InputManager<'a> {
    pub rows: [PinDriver<'a, Output>; 2],
    pub cols: [PinDriver<'a, Input>; 4],

    pub btn_joy_sw: PinDriver<'a, Input>,

    pub joy_x_pin: AnyInputPin<'a>,
    pub joy_y_pin: AnyInputPin<'a>,
    pub joy_rotation: JoystickRotation,

    pub key_states: KeyStates,
}

impl<'a> InputManager<'a> {
    pub fn build(
        keyboard: crate::pin_config::KeyboardMatrixConfig,
        joy: crate::pin_config::JoyPinConfig,
        rotation: JoystickRotation,
    ) -> Result<Self, anyhow::Error> {
        let [row0, row1] = keyboard.rows;

        let mut rows = [PinDriver::output(row0)?, PinDriver::output(row1)?];

        for row in &mut rows {
            row.set_high()?;
        }

        let [col0, col1, col2, col3] = keyboard.cols;

        let cols = [
            PinDriver::input(col0, Pull::Up)?,
            PinDriver::input(col1, Pull::Up)?,
            PinDriver::input(col2, Pull::Up)?,
            PinDriver::input(col3, Pull::Up)?,
        ];

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
            rows,
            cols,
            btn_joy_sw,
            joy_x_pin: joy.x,
            joy_y_pin: joy.y,
            joy_rotation: rotation,
            key_states: KeyStates::default(),
        })
    }

    pub fn scan(&mut self) -> Result<(), anyhow::Error> {
        let mut temp_states = [false; 8];

        for r in 0..2 {
            self.rows[r].set_low()?;

            unsafe {
                esp_idf_svc::sys::ets_delay_us(5);
            }

            for c in 0..4 {
                if self.cols[c].is_low() {
                    temp_states[r * 4 + c] = true;
                }
            }

            self.rows[r].set_high()?;
        }

        self.key_states.esc = temp_states[0];
        self.key_states.btn_1 = temp_states[1];
        self.key_states.btn_2 = temp_states[2];
        self.key_states.btn_3 = temp_states[3];
        self.key_states.btn_4 = temp_states[4];
        self.key_states.btn_5 = temp_states[5];
        self.key_states.btn_6 = temp_states[6];
        self.key_states.btn_7 = temp_states[7];

        Ok(())
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

    pub fn get_ui_events(&mut self, joy_data: JoystickData) -> UiEvents {
        self.key_states.get_events() | joy_data.get_events()
    }
}
