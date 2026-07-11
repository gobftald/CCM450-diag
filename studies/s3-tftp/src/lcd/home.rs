use core::str::from_utf8_unchecked as fuu;
use core::ptr::copy_nonoverlapping as cpn;

use lcd_async::{raw_framebuf::RawFrameBuf, Display};

use embedded_graphics::{
    image::ImageRaw,
    mono_font::{DecorationDimensions, MonoFont, MonoTextStyle, ascii::FONT_10X20, mapping::StrGlyphMapping},
    pixelcolor::{Rgb565, RgbColor}, prelude::Point, text::{Text, Baseline, TextStyleBuilder}
};
use embedded_graphics_core::{
    Drawable,
    draw_target::DrawTarget,
    geometry::Size,
};

use crate::{gps::GPS_DATA, gsm::GSM_OK, sd_card::SD_TOOK, tcp::TCP_STAT};
use super::DISPLAY_WIDTH;

const BIG_FONT_WIDTH: usize = 54;
pub const BIG_FONT_HEIGHT: usize = 100;
const BIG_FONT_SPACE: usize = 10;

const RPM_CHAR_NUM: usize = 4;
const RPM_TEXT_WIDTH: usize = RPM_CHAR_NUM * BIG_FONT_WIDTH + 3 * BIG_FONT_SPACE;

pub const SMALL_FONT_WIDTH: usize = 22;
const SMALL_FONT_HEIGHT: usize = 40;
pub const SMALL_FONT_SPACE: usize = 4;

pub const SLOW_TEXT_WIDTH: usize = DISPLAY_WIDTH;
pub const SLOW_TEXT_HEIGHT: usize = SMALL_FONT_HEIGHT;
pub const SLOW_FRAME_SIZE: usize = SLOW_TEXT_WIDTH * SLOW_TEXT_HEIGHT * super::PIXEL_SIZE;

const REFRESH_10_SEC: usize = 10000 / super::SCREEN_TICK;
const REFRESH_1_SEC: usize = 1000 / super::SCREEN_TICK;

pub const SMALL_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("small_font.raw"), 248),
    glyph_mapping: &StrGlyphMapping::new("0123456789a", 0),
    character_size: Size::new(SMALL_FONT_WIDTH as u32, SMALL_FONT_HEIGHT as u32),
    character_spacing: SMALL_FONT_SPACE as u32,
    baseline: 0,
    underline: DecorationDimensions::new(0, 0),
    strikethrough: DecorationDimensions::new(0, 0),
};

const BIG_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("big_font.raw"), 600),
    glyph_mapping: &StrGlyphMapping::new("0123456789a", 0),
    character_size: Size::new(BIG_FONT_WIDTH as u32, BIG_FONT_HEIGHT as u32),
    character_spacing: BIG_FONT_SPACE as u32,
    baseline: 0,
    underline: DecorationDimensions::new(0, 0),
    strikethrough: DecorationDimensions::new(0, 0),
};

pub struct HomeScreen {
    initial: bool,
    flip: u32,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            initial: true,
            flip: 0,
        }
    }

    #[allow(unused_assignments)]
    pub async fn update<DI, MODEL, RST>(
        &mut self,
        display: &mut Display<DI, MODEL, RST>,
        chunk_buffer: &mut [u8],
        mut tick: usize
    )
    where
        DI:    lcd_async::interface::Interface<Word = u8>,
        MODEL: lcd_async::models::Model<ColorFormat = Rgb565>,
        RST:   embedded_hal::digital::OutputPin,
    {
        // if screen changed to here, clear the whole display
        if self.initial {
            crate::lcd::clear_screen(display, chunk_buffer).await;
        }

        // const function
        let text_style = TextStyleBuilder::new().build();

        let mut fb = RawFrameBuf::<Rgb565, _>::new(
            &mut *chunk_buffer,
            DISPLAY_WIDTH / 2,
            BIG_FONT_HEIGHT
        );
        let _ = fb.clear(Rgb565::BLACK);

        // draw status flags
        let _ = Text::with_baseline(
                unsafe { fuu(&GSM_OK) },
                Point::new(0, 30),
                MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
                Baseline::Bottom,
            )
            .draw(&mut fb);

        let _ = Text::with_baseline(
                unsafe { fuu(&SD_TOOK) },
                Point::new(0, 60),
                MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
                Baseline::Bottom,
            )
            .draw(&mut fb);

        let _ = Text::with_baseline(
                unsafe { fuu(&TCP_STAT) },
                Point::new(0, 90),
                MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
                Baseline::Bottom,
            )
            .draw(&mut fb);
    
        // draw rpm value
        let character_style =
            // const function
            MonoTextStyle::new(&BIG_FONT, Rgb565::WHITE);
        
        // shifting digits test text
        let mut digits = *b"0123456789012345678";
        let rpm = &mut digits[tick % 10..tick % 10 + RPM_CHAR_NUM / 2];
        //rpm[RPM_CHAR_NUM -1 ] = b'0'; rpm[RPM_CHAR_NUM -2 ] = b'0';
        trail_space( rpm );
        let _ = Text::with_text_style(
            unsafe { fuu(rpm) },
            Point { x: ((DISPLAY_WIDTH - RPM_TEXT_WIDTH) / 2) as i32, y: 0 },
            character_style,
            text_style,
        )
        .draw(&mut fb);

        let _ = display.show_raw_data(
            0, 0,
            (DISPLAY_WIDTH / 2) as u16,
            BIG_FONT_HEIGHT as u16,
            chunk_buffer,
        ).await;

        // draw RPM second fix half only if screen changed
        if self.initial {
            let mut fb = RawFrameBuf::<Rgb565, _>::new(
                &mut *chunk_buffer,
                DISPLAY_WIDTH / 2,
                BIG_FONT_HEIGHT
            );

            let _ = fb.clear(Rgb565::BLACK);

            let _ = Text::with_text_style(
                unsafe { fuu(b"00") },
                Point { x: (BIG_FONT_SPACE / 2) as i32, y: 0 },
                character_style,
                text_style,
            )
            .draw(&mut fb);

            let _ = display.show_raw_data(
                (DISPLAY_WIDTH / 2) as u16,
                0,
                (DISPLAY_WIDTH / 2) as u16,
                BIG_FONT_HEIGHT as u16,
                chunk_buffer,
            ).await;

            self.initial = false;
            tick = 0; // to refresh both slow regions
        }

        // Force a voluntary yield point to let the async executor process the scheduler queue
        embassy_time::Timer::after_ticks(1).await;

        // coolant tmp, battery, air tmp
        
        let slow_buffer = &mut chunk_buffer[..SLOW_FRAME_SIZE];
        let character_style =
                MonoTextStyle::new(&SMALL_FONT, Rgb565::WHITE);

        if tick % REFRESH_10_SEC == 0 {
            let mut fb = RawFrameBuf::<Rgb565, _>::new(
                &mut *slow_buffer,
                SLOW_TEXT_WIDTH as usize,
                SLOW_TEXT_HEIGHT as usize
            );
            let _ = fb.clear(Rgb565::BLACK);

            // Coolant temperature

            let _ = Text::with_baseline(
                "T +",
                Point::new(0, 40),
                MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
                Baseline::Bottom,
            )
            .draw(&mut fb);

            let _ = Text::with_text_style(
                unsafe { fuu(b"00") },
                Point { x: 38, y: 0 },
                character_style,
                text_style,
            )
            .draw(&mut fb);

            // Battey voltage

            let _ = Text::with_baseline(
                "B ",
                Point::new(112, 40),
                MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
                Baseline::Bottom,
            )
            .draw(&mut fb);

            let _ = Text::with_text_style(
                unsafe { fuu(b"000") },
                Point { x: 132, y: 0 },
                character_style,
                text_style,
            )
            .draw(&mut fb);

            // Air temperature

            let _ = Text::with_baseline(
                "A +",
                Point::new(234, 40),
                MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE),
                Baseline::Bottom,
            )
            .draw(&mut fb);

            let _ = Text::with_text_style(
                unsafe { fuu(b"00") },
                Point { x: 272, y: 0 },
                character_style,
                text_style,
            )
            .draw(&mut fb);

            let _ = display.show_raw_data(
                0,
                130,
                SLOW_TEXT_WIDTH as u16,
                SLOW_TEXT_HEIGHT as u16,
                slow_buffer
            ).await;

            embassy_time::Timer::after_ticks(1).await;

        // COG, altitude, number of satelite  / hdop
        } else if tick % REFRESH_1_SEC == 0 {
            //debug!("{:a}", unsafe { core::slice::from_raw_parts(&raw const GPS_DATA as *const u8, 62) } );

            let mut fb = RawFrameBuf::<Rgb565, _>::new(
                &mut *slow_buffer,
                SLOW_TEXT_WIDTH as usize,
                SLOW_TEXT_HEIGHT as usize
            );
            let _ = fb.clear(Rgb565::BLACK);

            // COG
            let cog: &mut [u8] = &mut[ b'0', b'0', b'0'];
            unsafe { cpn(&raw const GPS_DATA.cog as *const u8, cog.as_mut_ptr(), 3) };
            trail_space( cog );
            let _ = Text::with_text_style(
                unsafe { fuu(cog) },
                Point { x: 0, y: 0 },
                character_style,
                text_style,
            )
            .draw(&mut fb);

            // Altitude
            let alt: &mut [u8] = &mut[ b'0', b'0', b'0', b'0'];
            unsafe { cpn(&raw const GPS_DATA.alt as *const u8, alt.as_mut_ptr(), 4) };
            trail_space( alt );
            let _ = Text::with_text_style(
                unsafe { fuu(alt) },
                Point { x: 97, y: 0 },
                character_style,
                text_style,
            )
            .draw(&mut fb);

            // Rotate through time / satellite count / hdop using self.flip
            if self.flip % 8 == 0 || self.flip % 8 == 1 || self.flip % 8 == 2
                || self.flip % 8 == 4 || self.flip % 8 == 5 || self.flip % 8 == 6 {
                // Time
                let time: &mut [u8] = &mut[ b'0', b'0', b'0', b'0'];
                unsafe { cpn(&raw const GPS_DATA.time as *const u8, time.as_mut_ptr(), 4) };
                if time[0] != b'h' || time[1] != b'h' || time[2] != b'm' || time[3] != b'm' { 
                    let _ = Text::with_text_style(
                        unsafe { fuu(time) },
                        Point { x: 220, y: 0 },
                        character_style,
                        text_style,
                    )
                    .draw(&mut fb);
                }
            } else if self.flip % 8 == 3 {
                // Number of satelite
                let sat: &mut [u8] = &mut[ b'0', b'0'];
                unsafe { cpn(&raw const GPS_DATA.sat as *const u8, sat.as_mut_ptr(), 2) };
                trail_space( sat );
                let _ = Text::with_text_style(
                    unsafe { fuu(sat) },
                    Point { x: 220, y: 0 },
                    character_style,
                    text_style,
                )
                .draw(&mut fb);
            } else {
                //  hdop
                let hdop: &mut [u8] = &mut[ b'0', b'0', b'0', b'0', b'0'];
                unsafe { cpn(&raw const GPS_DATA.hdop as *const u8, hdop.as_mut_ptr(), 5) };
                hdop[2] = hdop[3]; hdop[3] = hdop[4];
                trail_space( &mut hdop[..4] );
                let _ = Text::with_text_style(
                    unsafe { fuu(&hdop[..4]) },
                    Point { x: 220, y: 0 },
                    character_style,
                    text_style,
                )
                .draw(&mut fb);
            }
            self.flip += 1;

            let _ = display.show_raw_data(
                0,
                190,
                SLOW_TEXT_WIDTH as u16,
                SLOW_TEXT_HEIGHT as u16,
                slow_buffer
            ).await;

            embassy_time::Timer::after_ticks(1).await;
        }
    }

    pub fn reset(&mut self) { self.initial = true; }
}

fn trail_space(slice: &mut [u8]) {
    for c in slice {
        if *c == b'0' {
            *c = b'a'
        } else {
            break
        }
    }
}

/*
rpm 4

coolant temperature 2.5 -40 - 215 +/-00
battery 3.5 e.g. 11.7
air temperature 2.5 -40 - 215 +/-00
dtc 4 0000
-------------17
cog 3 000
alt 4 0000
sat 2 00
hdop 4 00.00
-------------17
*/
