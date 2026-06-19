use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{
    app_context::{AppContext, UpdateContext},
    apps::App,
    display::{Sh1106Unified, Ssd1306Unified, UnifiedDisplay},
    input::{InputManager, JoystickRotation},
    pin_config::{
        I2cDisplayPinConfig, JoyPinConfig, KeyboardMatrixConfig, PinConfig, RtcPinConfig,
    },
    rtc::{ds1302::Ds1302, sync_time},
    ui::UiEvents,
};
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_idf_svc::hal::{
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    units::KiloHertz,
};
use mini_oled::screen::sh1106::Sh1106;
use ssd1306::{I2CDisplayInterface, Ssd1306, mode::DisplayConfig, size::DisplaySize128x64};
use u8g2_fonts::{FontRenderer, fonts};

pub mod app_context;
pub mod apps;
pub mod display;
pub mod input;
pub mod pin_config;
pub mod rtc;
pub mod ui;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // NOTE: Change the GPIO config if you connected the hardware in different ports
    let pin_config = PinConfig {
        keyboard: KeyboardMatrixConfig {
            rows: [pins.gpio4.degrade_output(), pins.gpio5.degrade_output()],
            cols: [
                pins.gpio6.degrade_input(),
                pins.gpio16.degrade_input(),
                pins.gpio15.degrade_input(),
                pins.gpio13.degrade_input(),
            ],
        },
        i2c_display: I2cDisplayPinConfig {
            sda: pins.gpio9.degrade_input_output(),
            scl: pins.gpio8.degrade_input_output(),
            addr_1_3_in: 0x3D,
            addr_0_96_in: 0x3C,
        },
        rtc: RtcPinConfig {
            clk: pins.gpio10.degrade_output(),
            dat: pins.gpio11.degrade_input_output(),
            rst: pins.gpio12.degrade_output(),
        },
        joy: JoyPinConfig {
            x: pins.gpio3.degrade_input(),
            y: pins.gpio7.degrade_input(),
            sw: pins.gpio18.degrade_input(),
        },
    };

    let mut rtc_driver = {
        let pins = pin_config.rtc;
        let clk_pin = PinDriver::output(pins.clk)?;
        let dat_pin = PinDriver::input_output(pins.dat, esp_idf_svc::hal::gpio::Pull::Floating)?;
        let rst_pin = PinDriver::output(pins.rst)?;

        Ds1302::new(clk_pin, dat_pin, rst_pin)
    };
    rtc_driver.init()?;

    sync_time(&mut rtc_driver)?;

    let mut input_manager = InputManager::build(
        pin_config.keyboard,
        pin_config.joy,
        JoystickRotation::Deg270,
    )?;

    let i2c_cfg = I2cConfig::new().baudrate(KiloHertz::from(400).into());
    let i2c_driver = I2cDriver::new(
        peripherals.i2c0,
        pin_config.i2c_display.sda,
        pin_config.i2c_display.scl,
        &i2c_cfg,
    )?;

    let i2c_bus_shared = RefCell::new(i2c_driver);
    let i2c_dev_for_ssd = RefCellDevice::new(&i2c_bus_shared);
    let i2c_dev_for_sh = RefCellDevice::new(&i2c_bus_shared);

    let ssd_interface = I2CDisplayInterface::new_custom_address(
        i2c_dev_for_ssd,
        pin_config.i2c_display.addr_0_96_in,
    );
    let mut display_0_96 = Ssd1306::new(
        ssd_interface,
        DisplaySize128x64,
        ssd1306::prelude::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    display_0_96
        .init()
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let sh_interface =
        mini_oled::prelude::I2cInterface::new(i2c_dev_for_sh, pin_config.i2c_display.addr_1_3_in);
    let mut display_1_3 = Sh1106::new(sh_interface);
    display_1_3.init().map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let font = FontRenderer::new::<fonts::u8g2_font_6x10_tr>();
    let font_large = FontRenderer::new::<fonts::u8g2_font_10x20_tr>();

    let start_time = Instant::now();

    let mut last_tick = Instant::now();
    let target_frame_time = Duration::from_millis(33); // 20fps

    let mut active_app = App::Clock;

    let shared_events = Arc::new(Mutex::new(Vec::<UiEvents>::new()));
    let input_events = Arc::clone(&shared_events);

    loop {
        // get events
        let mut last_raw_events = UiEvents::empty();
        let joy_data = input_manager.read_joystick();
        input_manager.scan().unwrap();
        let current_raw_events = input_manager.get_ui_events(joy_data);
        log::info!("{:?}", current_raw_events);

        let just_pressed = current_raw_events & !last_raw_events;
        if !just_pressed.is_empty()
            && let Ok(mut events) = input_events.lock()
        {
            events.push(just_pressed);
        }

        last_raw_events = current_raw_events;

        // render loop
        let now = Instant::now();
        let elapsed = now.duration_since(last_tick);

        if elapsed >= target_frame_time {
            let frame_events = {
                if let Ok(mut events_guard) = shared_events.lock() {
                    std::mem::take(&mut *events_guard)
                } else {
                    Vec::new()
                }
            };

            if frame_events.is_empty() {
                let mut update_ctx = UpdateContext {
                    events: UiEvents::empty(),
                    rtc: &mut rtc_driver,
                };
                if let Some(new_app) = active_app.update(&mut update_ctx) {
                    active_app = new_app;
                }
            } else {
                for event in frame_events {
                    let mut update_ctx = UpdateContext {
                        events: event,
                        rtc: &mut rtc_driver,
                    };
                    if let Some(new_app) = active_app.update(&mut update_ctx) {
                        active_app = new_app;
                    }
                }
            }

            let mut app_ctx = AppContext {
                display_0_96: Ssd1306Unified::new(&mut display_0_96),
                display_1_3: Sh1106Unified::new(&mut display_1_3),
                font: &font,
                font_large: &font_large,
                uptime_secs: start_time.elapsed().as_secs(),
                input: &input_manager,
            };

            app_ctx.display_0_96.clear(BinaryColor::Off).ok();
            app_ctx.display_1_3.clear(BinaryColor::Off).ok();

            active_app.draw(&mut app_ctx)?;

            app_ctx.display_0_96.flush().ok();
            app_ctx.display_1_3.flush().ok();

            last_tick = Instant::now();
        }

        thread::sleep(Duration::from_millis(10));
    }
}
