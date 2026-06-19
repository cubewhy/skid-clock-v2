use embedded_graphics::{
    Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::BinaryColor,
};

use embedded_hal_bus::i2c::RefCellDevice;
use esp_idf_svc::hal::i2c::I2cDriver;
use mini_oled::{interface::i2c::I2cInterface, screen::sh1106::Sh1106};
use ssd1306::{
    Ssd1306, mode::BufferedGraphicsMode, prelude::I2CInterface, size::DisplaySize128x64,
};

use crate::ui::Rect;

pub type SharedI2cBus<'a> = RefCellDevice<'a, I2cDriver<'static>>;

pub type Display096<'a> = Ssd1306<
    I2CInterface<SharedI2cBus<'a>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub type Display13<'a> = Sh1106<I2cInterface<SharedI2cBus<'a>>>;

/// A unified interface for all OLED displays in the system.
pub trait UnifiedDisplay:
    DrawTarget<Color = BinaryColor, Error = core::convert::Infallible>
{
    /// Flush the RAM frame buffer to the physical screen.
    fn flush(&mut self) -> anyhow::Result<()>;

    /// Returns the resolution (width, height) of the display.
    fn resolution(&self) -> Size;

    fn rect(&self) -> Rect {
        let Size { width, height } = self.resolution();

        Rect::new(0, 0, width, height)
    }
}

pub struct Ssd1306Unified<'a, 'b> {
    display: &'a mut Display096<'b>,
}

impl<'a, 'b> Ssd1306Unified<'a, 'b> {
    pub fn new(display: &'a mut Display096<'b>) -> Self {
        Self { display }
    }
}

impl<'a, 'b> OriginDimensions for Ssd1306Unified<'a, 'b> {
    fn size(&self) -> Size {
        Size::new(128, 64)
    }
}

impl<'a, 'b> DrawTarget for Ssd1306Unified<'a, 'b> {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        // Draw directly to the underlying SSD1306 buffer
        self.display.draw_iter(pixels).ok();
        Ok(())
    }
}

impl<'a, 'b> UnifiedDisplay for Ssd1306Unified<'a, 'b> {
    fn flush(&mut self) -> anyhow::Result<()> {
        self.display
            .flush()
            .map_err(|e| anyhow::anyhow!("SSD1306 Flush Error: {:?}", e))
    }

    fn resolution(&self) -> Size {
        Size::new(128, 64)
    }
}

pub struct Sh1106Unified<'a, 'b> {
    display: &'a mut Display13<'b>,
}

impl<'a, 'b> Sh1106Unified<'a, 'b> {
    pub fn new(display: &'a mut Display13<'b>) -> Self {
        Self { display }
    }
}

impl<'a, 'b> OriginDimensions for Sh1106Unified<'a, 'b> {
    fn size(&self) -> Size {
        Size::new(128, 64)
    }
}

impl<'a, 'b> DrawTarget for Sh1106Unified<'a, 'b> {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.display.get_mut_canvas().draw_iter(pixels).ok();

        Ok(())
    }
}

impl<'a, 'b> UnifiedDisplay for Sh1106Unified<'a, 'b> {
    fn flush(&mut self) -> anyhow::Result<()> {
        self.display
            .flush()
            .map_err(|e| anyhow::anyhow!("SH1106 Flush Error: {:?}", e))
    }

    fn resolution(&self) -> Size {
        Size::new(128, 64)
    }
}
