use crate::rtc::UnifiedRtc;
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use esp_idf_svc::hal::gpio::{InputOutput, Output, PinDriver};

const REG_SECONDS_WRITE: u8 = 0x80;
const REG_SECONDS_READ: u8 = 0x81;
const REG_MINUTES_WRITE: u8 = 0x82;
const REG_MINUTES_READ: u8 = 0x83;
const REG_HOURS_WRITE: u8 = 0x84;
const REG_HOURS_READ: u8 = 0x85;
const REG_DATE_WRITE: u8 = 0x86;
const REG_DATE_READ: u8 = 0x87;
const REG_MONTH_WRITE: u8 = 0x88;
const REG_MONTH_READ: u8 = 0x89;
const REG_YEAR_WRITE: u8 = 0x8C;
const REG_YEAR_READ: u8 = 0x8D;
const REG_WP_WRITE: u8 = 0x8E;

const CLOCK_HALT_MASK: u8 = 0x7F;
const HOUR_24_MASK: u8 = 0x3F;
const WP_ENABLE: u8 = 0x80;
const WP_DISABLE: u8 = 0x00;

pub struct Ds1302<'a> {
    clk: PinDriver<'a, Output>,
    dat: PinDriver<'a, InputOutput>,
    rst: PinDriver<'a, Output>,
}

impl<'a> Ds1302<'a> {
    pub fn new(
        clk: PinDriver<'a, Output>,
        dat: PinDriver<'a, InputOutput>,
        rst: PinDriver<'a, Output>,
    ) -> Self {
        let mut s = Self { clk, dat, rst };
        s.clk.set_low().ok();
        s.rst.set_low().ok();
        s
    }

    /// Writes a single byte over the 3-wire bus
    fn set_dat_direction(&mut self, is_input: bool) {
        let mode = if is_input {
            esp_idf_svc::sys::gpio_mode_t_GPIO_MODE_INPUT
        } else {
            esp_idf_svc::sys::gpio_mode_t_GPIO_MODE_OUTPUT
        };
        unsafe {
            esp_idf_svc::sys::gpio_set_direction(self.dat.pin() as i32, mode);
        }
    }

    /// Writes a single byte over the 3-wire bus
    fn write_byte(&mut self, mut byte: u8) {
        for _ in 0..8 {
            let bit = (byte & 1) != 0;
            self.dat.set_level(bit.into()).ok();
            unsafe {
                esp_idf_svc::sys::ets_delay_us(2);
            }
            self.clk.set_high().ok();
            unsafe {
                esp_idf_svc::sys::ets_delay_us(2);
            }
            self.clk.set_low().ok();
            byte >>= 1;
        }
    }

    /// Reads a single byte over the 3-wire bus
    fn read_byte(&mut self) -> u8 {
        let mut byte = 0;
        for i in 0..8 {
            let bit = self.dat.is_high();
            if bit {
                byte |= 1 << i;
            }
            unsafe {
                esp_idf_svc::sys::ets_delay_us(2);
            }
            self.clk.set_high().ok();
            unsafe {
                esp_idf_svc::sys::ets_delay_us(2);
            }
            self.clk.set_low().ok();
        }
        byte
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        self.write_register(REG_WP_WRITE, WP_DISABLE)?;

        let seconds = self.read_register(REG_SECONDS_READ)?;
        self.write_register(REG_SECONDS_WRITE, seconds & CLOCK_HALT_MASK)?;
        Ok(())
    }

    pub fn read_register(&mut self, cmd: u8) -> anyhow::Result<u8> {
        self.set_dat_direction(false);
        self.rst.set_high().ok();
        unsafe {
            esp_idf_svc::sys::ets_delay_us(4);
        }

        self.write_byte(cmd);

        self.set_dat_direction(true);
        let val = self.read_byte();

        self.rst.set_low().ok();
        unsafe {
            esp_idf_svc::sys::ets_delay_us(4);
        }
        Ok(val)
    }

    pub fn write_register(&mut self, cmd: u8, val: u8) -> anyhow::Result<()> {
        self.set_dat_direction(false);
        self.rst.set_high().ok();
        unsafe {
            esp_idf_svc::sys::ets_delay_us(4);
        }

        self.write_byte(cmd);
        self.write_byte(val);

        self.rst.set_low().ok();
        unsafe {
            esp_idf_svc::sys::ets_delay_us(4);
        }
        Ok(())
    }
}

fn bcd_to_dec(bcd: u8) -> u8 {
    ((bcd & 0xF0) >> 4) * 10 + (bcd & 0x0F)
}

fn dec_to_bcd(dec: u8) -> u8 {
    ((dec / 10) << 4) | (dec % 10)
}

impl<'a> UnifiedRtc for Ds1302<'a> {
    fn read_time(&mut self) -> anyhow::Result<NaiveDateTime> {
        let sec = bcd_to_dec(self.read_register(REG_SECONDS_READ)? & CLOCK_HALT_MASK);
        let min = bcd_to_dec(self.read_register(REG_MINUTES_READ)?);
        let hour = bcd_to_dec(self.read_register(REG_HOURS_READ)? & HOUR_24_MASK);
        let day = bcd_to_dec(self.read_register(REG_DATE_READ)?);
        let month = bcd_to_dec(self.read_register(REG_MONTH_READ)?);
        let year = bcd_to_dec(self.read_register(REG_YEAR_READ)?) as i32 + 2000;

        let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
            .ok_or_else(|| anyhow::anyhow!("Invalid Date"))?;
        let time = NaiveTime::from_hms_opt(hour as u32, min as u32, sec as u32)
            .ok_or_else(|| anyhow::anyhow!("Invalid Time"))?;

        Ok(NaiveDateTime::new(date, time))
    }

    fn set_time(&mut self, time: &NaiveDateTime) -> anyhow::Result<()> {
        self.write_register(REG_WP_WRITE, WP_DISABLE)?;

        self.write_register(REG_SECONDS_WRITE, dec_to_bcd(time.second() as u8))?;
        self.write_register(REG_MINUTES_WRITE, dec_to_bcd(time.minute() as u8))?;
        self.write_register(REG_HOURS_WRITE, dec_to_bcd(time.hour() as u8))?;
        self.write_register(REG_DATE_WRITE, dec_to_bcd(time.day() as u8))?;
        self.write_register(REG_MONTH_WRITE, dec_to_bcd(time.month() as u8))?;

        let short_year = (time.year() - 2000) as u8;
        self.write_register(REG_YEAR_WRITE, dec_to_bcd(short_year))?;

        self.write_register(REG_WP_WRITE, WP_ENABLE)?;
        Ok(())
    }
}
