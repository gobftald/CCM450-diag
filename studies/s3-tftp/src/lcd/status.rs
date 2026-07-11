use core::str::from_utf8_unchecked as fuu;

use lcd_async::{raw_framebuf::RawFrameBuf, Display};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb565, RgbColor}, prelude::Point, text::{Text, TextStyleBuilder},
};
use embedded_graphics_core::{
    Drawable,
    draw_target::DrawTarget
};

use crate::lcd::home::{
    SLOW_TEXT_WIDTH, SLOW_TEXT_HEIGHT,
    SMALL_FONT_WIDTH, SMALL_FONT_SPACE,
    SLOW_FRAME_SIZE,
};

const DOT_SPACE: usize = 3;

pub struct StatusScreen {
    initial: bool,
    valid: bool,
}

impl StatusScreen {
    pub fn new() -> Self {
        Self {
            initial: true,
            valid: false,
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

        let bytes = unsafe { crate::tcp::TCP_ADDR.octets() };

        if bytes[0] != 0 && self.valid == false {
            let text_style = TextStyleBuilder::new().build();
            let character_style =
                MonoTextStyle::new(&crate::lcd::home::SMALL_FONT, Rgb565::WHITE);

            let mut fb = RawFrameBuf::<Rgb565, _>::new(
                &mut *chunk_buffer,
                SLOW_TEXT_WIDTH as usize,
                SLOW_TEXT_HEIGHT as usize
            );
            let _ = fb.clear(Rgb565::BLACK);

            let mut pos: usize = 0;
            for byte in unsafe { crate::tcp::TCP_ADDR.octets() } {
                let (buf, size) = byte_to_ascii_bytes(byte);

                trace!("*** status tcp bytes {:a} pos {}", &buf[..size], pos);
                let _ = Text::with_text_style(
                    unsafe { fuu(&buf[..size]) },
                    Point { x: pos as i32, y: 0 },
                    character_style,
                    text_style,
                )
                .draw(&mut fb);

                pos += (SMALL_FONT_WIDTH + SMALL_FONT_SPACE) * size + DOT_SPACE;
            }

            let _ = display.show_raw_data(
                0,
                0,
                SLOW_TEXT_WIDTH as u16,
                SLOW_TEXT_HEIGHT as u16,
                &chunk_buffer[..SLOW_FRAME_SIZE],
            ).await;
        
            self.valid = true;
        }
    }

    pub fn reset(&mut self) { self.initial = true; self.valid = false }
}

pub fn byte_to_ascii_bytes(byte: u8) -> ([u8; 3], usize) {
    let mut buf = [0u8; 3];
    let mut cursor = 0;
    
    if byte >= 100 {
        buf[0] = b'0' + (byte / 100);
        buf[1] = b'0' + ((byte / 10) % 10);
        buf[2] = b'0' + (byte % 10);
        cursor = 3;
    } else if byte >= 10 {
        buf[0] = b'0' + (byte / 10);
        buf[1] = b'0' + (byte % 10);
        cursor = 2;
    } else {
        buf[0] = b'0' + byte;
        cursor = 1;
    }

    (buf, cursor)
}
