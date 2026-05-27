use static_cell::StaticCell;

use esp_hal::gpio::{AnyPin, Input};

use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

use lcd_async::{
    interface::SpiInterface,
    raw_framebuf::RawFrameBuf,
    options::{Orientation, Rotation, ColorInversion},
};
use cst816s::{CST816S, TouchGesture};

use embedded_graphics::pixelcolor::Rgb565;

#[macro_use]
mod macros;

mod screen;
mod home;
mod graph;
mod logs;

use screen::{Screen, draw_screen};

// Display parameters
const WIDTH: u16 = 240;
const HEIGHT: u16 = 320;
const PIXEL_SIZE: usize = 2; // RGB565 = 2 bytes per pixel
const FRAME_SIZE: usize = (WIDTH as usize) * (HEIGHT as usize) * PIXEL_SIZE;

static FRAME_BUFFER: StaticCell<[u8; FRAME_SIZE]> = StaticCell::new();

#[embassy_executor::task]
pub(crate) async fn lcd_task(
    i2c_bus: esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>,
    mut tp_int: Input<'static>,
    spi_bus: &'static crate::SharedSpiBus,
    sd_ready: &'static Signal<NoopRawMutex, ()>,
    cs_pin: AnyPin<'static>,
    dc_pin: AnyPin<'static>,
    reset_pin: AnyPin<'static>,
    backlight_pin: AnyPin<'static>,
) {
    // WAIT for the SD card to finish its handshake
    sd_ready.wait().await;

    // Initialize lcd display
    let mut display =
        lcd_init!(spi_bus, cs_pin, dc_pin, reset_pin, backlight_pin);
    
    // Initialize touch controller
    create_no_input_pin!();
    let mut touch = CST816S::new(
        i2c_bus,
        NoInputPin, // so we can further use it for wait_for_falling_edge()
        esp_hal::gpio::NoPin
    );

    info!("Display initialized!");

    let frame_buffer = FRAME_BUFFER.init_with(|| [0; FRAME_SIZE]);

    let mut current_screen = Screen::Graph;
    let mut dirty = true; // only redraw when needed
    let mut inc: i32 = 0;

    loop {
        if dirty {
            let mut raw_fb =
                RawFrameBuf::<Rgb565, _>::new(
                    frame_buffer.as_mut_slice(), WIDTH.into(), HEIGHT.into()
                );

            draw_screen(&mut raw_fb, current_screen, inc);

            // Send the framebuffer data to the display
            display
                .show_raw_data(0, 0, WIDTH, HEIGHT, frame_buffer)
                .await
                .unwrap();

            dirty = false;
        }

        // wait for either touch interrupt OR 200ms timeout
        match embassy_time::with_timeout(
            embassy_time::Duration::from_millis(200),
            tp_int.wait_for_falling_edge()
        ).await {
            Ok(_) => {
                // interrupt fired — touch event ready
                match touch.read_one_touch_event(false) {  // false = don't check pin, we know it fired
                    Some(event) => {
                        /*
                        match event.action {
                            0 => debug!("Touch pressed at: ({}, {})", event.x, event.y),
                            1 => debug!("Touch released at: ({}, {})", event.x, event.y),
                            2 => debug!("Touch contact at: ({}, {})", event.x, event.y),
                            _ => {}
                        }
                        */
                        match event.gesture {
                            TouchGesture::SlideUp => {
                                debug!("Gesture: Slide Up");
                                current_screen = current_screen.prev();
                                dirty = true;
                            }
                            TouchGesture::SlideDown => {
                                debug!("Gesture: Slide Down");
                                current_screen = current_screen.next();
                                dirty = true;
                            }
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
                // only animate screens that need it
                if current_screen == Screen::Graph {
                    inc += 4;
                    dirty = true;
                }
            }
        }
    }
}
