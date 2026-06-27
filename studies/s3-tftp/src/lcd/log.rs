use lcd_async::{raw_framebuf::RawFrameBuf, Display};

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::RgbColor,
};

use super::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

pub struct LogScreen {
    initial: bool
}

impl LogScreen {
    pub fn new() -> Self {
        Self {
            initial: true
        }
    }

    pub async fn update<DI, MODEL, RST>(
        &mut self,
        display: &mut Display<DI, MODEL, RST>,
        full_buffer: &mut [u8],
    )
    where
        DI:    lcd_async::interface::Interface<Word = u8>,
        MODEL: lcd_async::models::Model<ColorFormat = Rgb565>,
        RST:   embedded_hal::digital::OutputPin,
    {
        let mut fb = RawFrameBuf::<Rgb565, _>::new(
            &mut *full_buffer,
            DISPLAY_WIDTH as usize,
            DISPLAY_HEIGHT as usize
        );
        let _ = fb.clear(Rgb565::BLACK);
        
        let _ = display
        .show_raw_data(
            0,
            0,
            DISPLAY_WIDTH as u16,
            DISPLAY_HEIGHT as u16,
            full_buffer)
        .await;
    }

    pub fn reset(&mut self) { self.initial = true; }
}
