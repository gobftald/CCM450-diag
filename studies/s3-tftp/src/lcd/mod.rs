use static_cell::StaticCell;

use esp_hal::{
    //time::Instant,
    gpio::{AnyPin, Input, Level, Output, OutputConfig}
};
use embassy_time::Instant;

use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

use lcd_async::{
    interface::SpiInterface,
    options::{Orientation, Rotation, ColorInversion},
};
use cst816s::{CST816S, TouchGesture};

#[macro_use]
mod macros;

mod home;
mod graph;
mod log;

pub const SCREEN_TICK: usize = 200; // ms

// Display parameters
pub const DISPLAY_WIDTH: usize = 320;
pub const DISPLAY_HEIGHT: usize = 240;
const PIXEL_SIZE: usize = 2; // RGB565 = 2 bytes per pixel
const DISPLAY_FRAME_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT * PIXEL_SIZE;

// Force the compiler to bind the starting address to 32 bytes block of cache lines
// The CPU accesses PSRAM through this cache.
#[repr(align(32))]
struct AlignedBuffer([u8; DISPLAY_FRAME_SIZE]);

// it is a large buffer, so put it into PSRAM
#[unsafe(link_section = ".ext_ram.bss")]
static DISPLAY_FRAME_BUFFER: StaticCell<AlignedBuffer> = StaticCell::new();

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
        lcd_init!(spi_bus, cs_pin, dc_pin, reset_pin);
    
    // Initialize touch controller
    create_no_input_pin!();
    let mut touch = CST816S::new(
        i2c_bus,
        NoInputPin, // so we can further use it for wait_for_falling_edge()
        esp_hal::gpio::NoPin
    );

    info!("Display initialized!");

    let display_frame_buffer =
    &mut DISPLAY_FRAME_BUFFER.init_with(|| AlignedBuffer([0; DISPLAY_FRAME_SIZE])).0;

    let mut home = home::HomeScreen::new();
    let mut graph = graph::GraphScreen::new();
    let mut log = log::LogScreen::new();

    let mut current = Screen::Home;
    let mut refresh = true;
    let mut tick = 0;

    home.update(&mut display, display_frame_buffer, tick).await;
    Output::new(backlight_pin, Level::High, OutputConfig::default());

    loop {
        if refresh {
            debug!("SOF lcd refresh");
            match current {
                Screen::Home  => home.update(&mut display, display_frame_buffer, tick).await,
                Screen::Graph => graph.update(&mut display, display_frame_buffer, tick).await,
                Screen::Log => log.update(&mut display, display_frame_buffer).await,
            }
            debug!("EOF lcdrefresh");
        } else {
            refresh = true;
        }

        // wait for either touch interrupt OR 200ms timeout
        match embassy_time::with_timeout(
            embassy_time::Duration::from_millis(SCREEN_TICK as u64),
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
                        current = match event.gesture {
                            TouchGesture::SlideUp => {
                                debug!("Gesture: Slide Up");
                                current.prev()
                            }
                            TouchGesture::SlideDown => {
                                debug!("Gesture: Slide Down");
                                current.next()
                            }
                            /*
                            TouchGesture::SlideLeft => debug!("Gesture: Slide Left"),
                            TouchGesture::SlideRight => debug!("Gesture: Slide Right"),
                            TouchGesture::SingleClick => debug!("Gesture: Single Click"),
                            TouchGesture::DoubleClick => debug!("Gesture: Double Click"),
                            TouchGesture::LongPress => debug!("Gesture: Long Press"),
                            TouchGesture::None => {}
                            */
                            _ => {
                                refresh = false;
                                current
                            }
                        };

                        // reset the screen we're navigating to
                        match current {
                            Screen::Home  => home.reset(),
                            Screen::Graph => graph.reset(),
                            Screen::Log => log.reset(),
                        }
                    }
                    None => {
                        refresh = false;
                    }
                }
            }
            Err(_) => {
                tick += 4;
            }
        }
    }
}

pub enum Screen {
    Home,
    Graph,
    Log,
}

impl Screen {
    pub fn next(self) -> Self {
        match self {
            Screen::Home => Screen::Graph,
            Screen::Graph => Screen::Log,
            Screen::Log => Screen::Home,   // wrap around
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Screen::Home => Screen::Log,
            Screen::Graph => Screen::Home,
            Screen::Log => Screen::Graph,
        }
    }
}
