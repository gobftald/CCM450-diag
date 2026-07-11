use lcd_async::{raw_framebuf::RawFrameBuf, Display};

use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::{RgbColor, Point, Primitive, Drawable},
    primitives::{Circle, Triangle, PrimitiveStyle},
};

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
        chunk_buffer: &mut [u8],
        tick: usize,
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

        let tick = tick * 2;

        {
            if tick % 80 == 0 {
                let mut fb = RawFrameBuf::<Rgb565, _>::new(
                    &mut chunk_buffer[16000..],
                    80, 100,
                );

                let _ = fb.clear(Rgb565::BLACK);

                let _ = display
                    .show_raw_data(
                        80 + 78, 80,
                        80, 100,
                        &chunk_buffer[16000..])
                    .await;
            } else {
                let mut fb = RawFrameBuf::<Rgb565, _>::new(
                    &mut chunk_buffer[16000..],
                    1, 100,
                );
                let _ = fb.clear(Rgb565::BLACK);

                let _ = display
                .show_raw_data(
                    78 + (tick % 80) as u16, 80,
                    2, 100,
                    &chunk_buffer[16000..16000 + 400])
                .await;
            }
        }

        let mut fb = RawFrameBuf::<Rgb565, _>::new(
            &mut *chunk_buffer,
            80, 100,
        );

        let _ = fb.clear(Rgb565::BLACK);

        let _ = draw_smiley(&mut fb);

        let _ = display
            .show_raw_data(
                80 + (tick % 80) as u16, 80,
                80, 100,
                &chunk_buffer[..16000])
            .await;
    }

    pub fn reset(&mut self) { self.initial = true; }
}

fn draw_smiley<T>(raw_fb: &mut T) -> Result<(), T::Error>
where
    T: DrawTarget<Color = Rgb565>,
{
    // Draw the left eye as a circle located at (80, 80), with a diameter of 30, filled with white
    Circle::new(Point::new(0, 0), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(raw_fb)?;

    // Draw the right eye as a circle located at (130, 80), with a diameter of 30, filled with white
    Circle::new(Point::new(50, 0), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(raw_fb)?;

    // Draw an upside down triangle to represent a smiling mouth
    Triangle::new(
        Point::new(0, 60),  // Left point
        Point::new(80, 60), // Right point
        Point::new(40, 100), // Bottom point
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(raw_fb)?;

    // Cover the top part of the mouth with a black triangle so it looks like a smile
    Triangle::new(
        Point::new(10, 70), // Left point
        Point::new(70, 70), // Right point
        Point::new(40, 90), // Bottom point
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(raw_fb)?;

    Ok(())
}
