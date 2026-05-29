use lcd_async::{raw_framebuf::RawFrameBuf, Display};

use embedded_graphics::{
    image::ImageRaw, mono_font::{DecorationDimensions, MonoFont, MonoTextStyle, mapping::StrGlyphMapping}, pixelcolor::{Rgb565, RgbColor}, text::{Alignment, Baseline, Text, TextStyleBuilder}
};
use embedded_graphics_core::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Size, Dimensions}
};

const RPM_TEXT_WIDTH: u16 = 320;
const RPM_TEXT_HEIGHT: u16 = 62;
use super::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

const SEVENT_SEGMENT_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("big_font.raw"), 224),
    glyph_mapping: &StrGlyphMapping::new("0123456789", 0),
    character_size: Size::new(22, 40),
    character_spacing: 4,
    baseline: 7,
    underline: DecorationDimensions::default_underline(40),
    strikethrough: DecorationDimensions::default_strikethrough(40),
};

pub struct HomeScreen {
    rpm_buffer: [u8; RPM_TEXT_WIDTH as usize * RPM_TEXT_HEIGHT as usize * 2],
    initial: bool
}
impl HomeScreen {
    pub fn new() -> Self {
        Self {
            rpm_buffer: [0u8; (RPM_TEXT_WIDTH * RPM_TEXT_HEIGHT * 2) as usize],
            initial:    true,
        }
    }

    pub async fn update<DI, MODEL, RST>(
        &mut self,
        display: &mut Display<DI, MODEL, RST>,
        full_buffer: &mut [u8],
        incr: i32
    )
    where
        DI:    lcd_async::interface::Interface<Word = u8>,
        MODEL: lcd_async::models::Model<ColorFormat = Rgb565>,
        RST:   embedded_hal::digital::OutputPin,
    {
        // if screen changed to here, clear the display
        if self.initial {
            let mut fb = RawFrameBuf::<Rgb565, _>::new(
                &mut *full_buffer,
                DISPLAY_WIDTH as usize,
                DISPLAY_HEIGHT as usize
            );

            fb.clear(Rgb565::BLACK).unwrap();

            display.show_raw_data(
                0,
                0,
                DISPLAY_WIDTH,
                DISPLAY_HEIGHT,
                full_buffer
            ).await.unwrap();

            self.initial = false;
        }

        // draw rpm value

        let character_style =
            // const function
            MonoTextStyle::new(&SEVENT_SEGMENT_FONT, Rgb565::WHITE);

        // const function
        let text_style = TextStyleBuilder::new()
            .baseline(Baseline::Alphabetic)
            .alignment(Alignment::Center)
            .build();

        let mut fb = RawFrameBuf::<Rgb565, _>::new(
            self.rpm_buffer.as_mut_slice(),
            RPM_TEXT_WIDTH as usize,
            RPM_TEXT_HEIGHT as usize
        );

        fb.clear(Rgb565::BLACK).unwrap();

        // shifting digits test text
        if let Some(slice) = "0123456789012345678".get(
            ((incr/4)%10) as usize..(((incr/4)%10)+10) as usize) {
            Text::with_text_style(
                slice,
                fb.bounding_box().center(),
                character_style,
                text_style,
            )
            .draw(&mut fb)
            .unwrap();
        }

        display.show_raw_data(
            0,
            0,
            RPM_TEXT_WIDTH,
            RPM_TEXT_HEIGHT,
            self.rpm_buffer.as_mut_slice()
        ).await.unwrap();
    }

    pub fn reset(&mut self) { self.initial = true; }
}

/*
rpm 4

coolant temperature 3 -40 - 215 +/-00
battery 4 e.g. 11.7
air temperature 3 -40 - 215 +/-00
dtc 4 0000
-------------17
cog 3 000
alt 4 0000
sat 2 00
hdop 5 00.00
-------------17
*/
