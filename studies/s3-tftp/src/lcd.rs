use static_cell::StaticCell;

use esp_hal::{
    gpio::{self, AnyPin, Level, Input, Output, OutputConfig},
    spi::master::Config as SpiConfig,
};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;

use crate::SharedSpiBus;

use lcd_async::{
    interface::SpiInterface,
    raw_framebuf::RawFrameBuf,
    options::{Orientation, Rotation, ColorInversion},
};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::{RgbColor, Point, Primitive, Drawable},
    primitives::{Circle, Triangle, PrimitiveStyle},
};
use cst816s::{CST816S, TouchGesture};

// Display parameters
const WIDTH: u16 = 240;
const HEIGHT: u16 = 320;
const PIXEL_SIZE: usize = 2; // RGB565 = 2 bytes per pixel
const FRAME_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) * PIXEL_SIZE;

static FRAME_BUFFER: StaticCell<[u8; FRAME_SIZE]> = StaticCell::new();

pub struct NoInputPin;

impl embedded_hal::digital::ErrorType for NoInputPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::InputPin for NoInputPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> { Ok(true) }
    fn is_low(&mut self) -> Result<bool, Self::Error> { Ok(false) }
}

#[embassy_executor::task]
pub(crate) async fn lcd_task(
    i2c_bus: esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>,
    mut tp_int: Input<'static>,
    spi_bus: &'static SharedSpiBus,
    sd_ready: &'static Signal<NoopRawMutex, ()>,
    cs_pin: AnyPin<'static>,
    dc_pin: AnyPin<'static>,
    reset_pin: AnyPin<'static>,
    backlight_pin: AnyPin<'static>,
) {
    // WAIT for the SD card to finish its handshake
    sd_ready.wait().await;

    let cs = Output::new(cs_pin, Level::High, OutputConfig::default());
    let dc = Output::new(dc_pin, Level::Low, OutputConfig::default());
    let reset = Output::new(reset_pin, Level::High, OutputConfig::default());
    Output::new(backlight_pin, Level::High, OutputConfig::default());

    let config = SpiConfig::default()
        //.with_frequency(esp_hal::time::Rate::from_mhz(20))
        .with_frequency(esp_hal::time::Rate::from_mhz(20))
        .with_mode(esp_hal::spi::Mode::_0);

    let device = SpiDeviceWithConfig::new(spi_bus, cs, config);
    let di = SpiInterface::new(device, dc);
    
    let mut display =
        lcd_async::Builder::new(lcd_async::models::ST7789, di)
            .reset_pin(reset)
            .display_size(WIDTH as u16, HEIGHT as u16)
            .orientation(Orientation {
                rotation: Rotation::Deg0,
                mirrored: false,
            })
            .display_offset(0, 0)
            .invert_colors(ColorInversion::Inverted)
            .init(&mut embassy_time::Delay)
            .await
            .unwrap();
    
    // Initialize touch controller
    let mut touch = CST816S::new(
        i2c_bus,
        NoInputPin, // so we can further use it for wait_for_falling_edge()
        gpio::NoPin
    );
    
    info!("Display initialized!");

    let frame_buffer = FRAME_BUFFER.init_with(|| [0; FRAME_SIZE]);
    let mut inc: i32 = 0;
    loop {
        let mut raw_fb =
                RawFrameBuf::<Rgb565, _>::new(frame_buffer.as_mut_slice(), WIDTH.into(), HEIGHT.into());
            raw_fb.clear(Rgb565::BLACK).unwrap();

        // Draw a simple smiley face
        draw_smiley(&mut raw_fb, inc).unwrap();

        // Send the framebuffer data to the display
        display
            .show_raw_data(0, 0, WIDTH, HEIGHT, frame_buffer)
            .await
            .unwrap();

        // wait for either touch interrupt OR 200ms timeout
        match embassy_time::with_timeout(
            embassy_time::Duration::from_millis(200),
            tp_int.wait_for_falling_edge()
        ).await {
            Ok(_) => {
                // interrupt fired — touch event ready
                match touch.read_one_touch_event(false) {  // false = don't check pin, we know it fired
                    Some(event) => {
                        match event.action {
                            0 => debug!("Touch pressed at: ({}, {})", event.x, event.y),
                            1 => debug!("Touch released at: ({}, {})", event.x, event.y),
                            2 => debug!("Touch contact at: ({}, {})", event.x, event.y),
                            _ => {}
                        }
                        match event.gesture {
                            TouchGesture::SlideUp => debug!("Gesture: Slide Up"),
                            TouchGesture::SlideDown => debug!("Gesture: Slide Down"),
                            TouchGesture::SlideLeft => debug!("Gesture: Slide Left"),
                            TouchGesture::SlideRight => debug!("Gesture: Slide Right"),
                            TouchGesture::SingleClick => debug!("Gesture: Single Click"),
                            TouchGesture::DoubleClick => debug!("Gesture: Double Click"),
                            TouchGesture::LongPress => debug!("Gesture: Long Press"),
                            TouchGesture::None => {}
                        }
                    }
                    None => {}
                }
            }
            Err(_) => {
                // timeout — no touch, just continue drawing
                inc += 4;
            }
        }
    }
}

/*
ST7789T3 display driver
CST816D Capacitive Touch // GT911 driver communicates with the Goodix GT911 touch controller
QMI8658 6-axis IMU
*/

fn draw_smiley<T>(display: &mut T, inc: i32) -> Result<(), T::Error>
where
    T: DrawTarget<Color = Rgb565>,
{
    // Draw the left eye as a circle located at (80, 80), with a diameter of 30, filled with white
    Circle::new(Point::new(80 + (inc % 20), 80), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw the right eye as a circle located at (130, 80), with a diameter of 30, filled with white
    Circle::new(Point::new(130 + (inc % 20), 80), 30)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw an upside down triangle to represent a smiling mouth
    Triangle::new(
        Point::new(80 + (inc % 20), 140),  // Left point
        Point::new(160 + (inc % 20), 140), // Right point
        Point::new(120 + (inc % 20), 180), // Bottom point
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(display)?;

    // Cover the top part of the mouth with a black triangle so it looks like a smile
    Triangle::new(
        Point::new(90 + (inc % 20), 150),  // Left point
        Point::new(150 + (inc % 20), 150), // Right point
        Point::new(120 + (inc % 20), 170), // Bottom point
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(display)?;

    Ok(())
}