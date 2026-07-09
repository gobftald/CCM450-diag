use lcd_async::Display;
use embedded_graphics::pixelcolor::Rgb565;

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
        chunk_buffer: &mut [u8],
    )
    where
        DI:    lcd_async::interface::Interface<Word = u8>,
        MODEL: lcd_async::models::Model<ColorFormat = Rgb565>,
        RST:   embedded_hal::digital::OutputPin,
    {
        // if screen changed to here, clear the whole display
        if self.initial {
            crate::lcd::clear_screen(display, chunk_buffer).await;
            self.initial = false;
        }
    }

    pub fn reset(&mut self) { self.initial = true; }
}
