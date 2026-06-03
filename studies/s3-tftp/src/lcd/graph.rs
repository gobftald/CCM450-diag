use lcd_async::{raw_framebuf::RawFrameBuf, Display};

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::{RgbColor, Point, Primitive, Drawable},
    primitives::{Circle, Triangle, PrimitiveStyle},
};

use super::{DISPLAY_WIDTH, DISPLAY_HEIGHT};

pub struct GraphScreen {
    initial: bool
}

impl GraphScreen {
    pub fn new() -> Self {
        Self {
            initial: true
        }
    }

    pub async fn update<DI, MODEL, RST>(
        &mut self,
        display: &mut Display<DI, MODEL, RST>,
        full_buffer: &mut [u8],
        tick: usize,
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

        fb.clear(Rgb565::BLACK).ok();

        draw_smiley(&mut fb, tick).ok();

        display
            .show_raw_data(
                0,
                0,
                DISPLAY_WIDTH as u16,
                DISPLAY_HEIGHT as u16,
                full_buffer)
            .await.ok();
    }

    pub fn reset(&mut self) { self.initial = true; }
}

fn draw_smiley<T>(raw_fb: &mut T, tick: usize) -> Result<(), T::Error>
where
    T: DrawTarget<Color = Rgb565>,
{
    const SLIDE: usize = 80;

    // Draw the left eye as a circle located at (80, 80), with a diameter of 30, filled with white
    Circle::new(Point::new(80 + (tick % SLIDE) as i32, 80), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(raw_fb)?;

    // Draw the right eye as a circle located at (130, 80), with a diameter of 30, filled with white
    Circle::new(Point::new(130 + (tick % SLIDE) as i32, 80), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(raw_fb)?;

    // Draw an upside down triangle to represent a smiling mouth
    Triangle::new(
        Point::new(80 + (tick % SLIDE) as i32, 140),  // Left point
        Point::new(160 + (tick % SLIDE) as i32, 140), // Right point
        Point::new(120 + (tick % SLIDE) as i32, 180), // Bottom point
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(raw_fb)?;

    // Cover the top part of the mouth with a black triangle so it looks like a smile
    Triangle::new(
        Point::new(90 + (tick % SLIDE) as i32, 150),  // Left point
        Point::new(150 + (tick % SLIDE) as i32, 150), // Right point
        Point::new(120 + (tick % SLIDE) as i32, 170), // Bottom point
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(raw_fb)?;

    Ok(())
}
