use embedded_graphics::{
    pixelcolor::{Rgb565, RgbColor},
    image::ImageRaw,
    mono_font::{mapping::StrGlyphMapping, DecorationDimensions, MonoFont, MonoTextStyle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics_core::{
    Drawable,
    geometry::{Size, Dimensions}
};

use lcd_async::raw_framebuf::RawFrameBuf;

const SEVENT_SEGMENT_FONT: MonoFont = MonoFont {
    image: ImageRaw::new(include_bytes!("big_font.raw"), 224),
    glyph_mapping: &StrGlyphMapping::new("0123456789", 0),
    character_size: Size::new(22, 40),
    character_spacing: 4,
    baseline: 7,
    underline: DecorationDimensions::default_underline(40),
    strikethrough: DecorationDimensions::default_strikethrough(40),
};

pub fn draw_home(fb: &mut RawFrameBuf<Rgb565, &mut [u8]>) {
    let character_style =
        MonoTextStyle::new(&SEVENT_SEGMENT_FONT, Rgb565::WHITE);
    let text_style = TextStyleBuilder::new()
        .baseline(Baseline::Bottom)
        .alignment(Alignment::Center)
        .build();

        Text::with_text_style(
            "123456",
            fb.bounding_box().center(),
            character_style,
            text_style,
        )
        .draw(fb)
        .unwrap();
}
