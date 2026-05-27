use lcd_async::raw_framebuf::RawFrameBuf;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::{RgbColor, Point, Primitive, Drawable},
    primitives::{Circle, Triangle, PrimitiveStyle},
};

pub fn draw_graph(fb: &mut RawFrameBuf<Rgb565, &mut [u8]>, incr: i32) {
    draw_smiley(fb, incr).unwrap();
}

fn draw_smiley<T>(raw_fb: &mut T, inc: i32) -> Result<(), T::Error>
where
    T: DrawTarget<Color = Rgb565>,
{
    const SLIDE: i32 =80;

    // Draw the left eye as a circle located at (80, 80), with a diameter of 30, filled with white
    Circle::new(Point::new(80 + (inc % SLIDE), 80), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(raw_fb)?;

    // Draw the right eye as a circle located at (130, 80), with a diameter of 30, filled with white
    Circle::new(Point::new(130 + (inc % SLIDE), 80), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(raw_fb)?;

    // Draw an upside down triangle to represent a smiling mouth
    Triangle::new(
        Point::new(80 + (inc % SLIDE), 140),  // Left point
        Point::new(160 + (inc % SLIDE), 140), // Right point
        Point::new(120 + (inc % SLIDE), 180), // Bottom point
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(raw_fb)?;

    // Cover the top part of the mouth with a black triangle so it looks like a smile
    Triangle::new(
        Point::new(90 + (inc % SLIDE), 150),  // Left point
        Point::new(150 + (inc % SLIDE), 150), // Right point
        Point::new(120 + (inc % SLIDE), 170), // Bottom point
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(raw_fb)?;

    Ok(())
}
