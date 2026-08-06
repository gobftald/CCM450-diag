use static_cell::StaticCell;

use esp_hal::gpio::{AnyPin, Input, Level, Output, OutputConfig};

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};

use lcd_async::{
    interface::SpiInterface,
    options::{Orientation, Rotation, ColorInversion},
};
use cst816s::{CST816S, TouchGesture};

#[macro_use]
mod macros;

mod home;
mod graph;
mod status;

// it is a delta after screen refresh finished - so refresh time (typical 30 - 50 ms) is added
pub const SCREEN_TICK: usize = 200; // ms

// Display parameters
pub const DISPLAY_WIDTH: usize = 320;
pub const DISPLAY_HEIGHT: usize = 240;
pub const PIXEL_SIZE: usize = 2; // RGB565 = 2 bytes per pixel

pub const CHUNK_FRAME_SIZE: usize = home::BIG_FONT_HEIGHT * DISPLAY_WIDTH * PIXEL_SIZE / 2;
pub const INIT_BAND_HEIGHT: usize = CHUNK_FRAME_SIZE / PIXEL_SIZE / DISPLAY_WIDTH;

// Force the compiler to bind the starting address to 32 bytes block of cache lines
// The CPU accesses PSRAM through this cache.
#[repr(align(32))]
struct AlignedBuffer([u8; CHUNK_FRAME_SIZE]);

// it is a large buffer, so put it into PSRAM
#[unsafe(link_section = ".ext_ram.bss")]
static CHUNK_FRAME_BUFFER: StaticCell<AlignedBuffer> = StaticCell::new();

pub(crate) async fn lcd_task(
    i2c_bus: &mut esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>,
    mut tp_int: Input<'static>,
    spi_bus: &'static crate::SharedSpiBus,
    wifi_rescan_request: &'static Signal<NoopRawMutex, ()>,
    cs_pin: Output<'static>,
    dc_pin: AnyPin<'static>,
    reset_pin: AnyPin<'static>,
    backlight_pin: AnyPin<'static>,
) {
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

    let chunk_frame_buffer =
        &mut CHUNK_FRAME_BUFFER.init_with(|| AlignedBuffer([0; CHUNK_FRAME_SIZE])).0;

    let mut home = home::HomeScreen::new();
    let mut graph = graph::GraphScreen::new();
    let mut status = status::StatusScreen::new();

    let mut current = Screen::Home;
    let mut long_press_handled = false;
    let mut slide_right = false;
    let mut refresh = true;
    let mut tick = 0;

    home.update(&mut display, chunk_frame_buffer, tick).await;
    Output::new(backlight_pin, Level::High, OutputConfig::default());

    loop {
        if refresh {
            trace!("SOF lcd refresh");
            match current {
                Screen::Home  => home.update(&mut display, chunk_frame_buffer, tick).await,
                Screen::Graph => graph.update(&mut display, chunk_frame_buffer, tick).await,
                Screen::Status => status.update(&mut display, chunk_frame_buffer).await,
            }
            trace!("EOF lcdrefresh");
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
                                if slide_right {
                                    debug!("go to sleep");
                                    break;
                                }
                                current.prev()
                            }
                            TouchGesture::SlideDown => {
                                debug!("Gesture: Slide Down");
                                current.next()
                            }
                            TouchGesture::LongPress => {
                                debug!("Gesture: LongPress");
                                if current == Screen::Status && !long_press_handled {
                                    wifi_rescan_request.signal(());
                                    long_press_handled = true;
                                    debug!("wifi_rescan_request.signal(())");
                                }
                                current
                            }
                            TouchGesture::SlideRight => {
                                debug!("Gesture: Slide Right");
                                if !slide_right {
                                    slide_right = true;
                                }
                                current
                            }
                            /*
                            TouchGesture::SlideLeft => debug!("Gesture: Slide Left"),
                            TouchGesture::SingleClick => debug!("Gesture: Single Click"),
                            TouchGesture::DoubleClick => debug!("Gesture: Double Click"),
                            TouchGesture::LongPress => debug!("Gesture: Long Press"),
                            TouchGesture::None => {}
                            */
                            _ => {
                                long_press_handled = false;
                                refresh = false;
                                current
                            }
                        };

                        // reset the screen we're navigating to
                        match current {
                            Screen::Home  => home.reset(),
                            Screen::Graph => graph.reset(),
                            Screen::Status => status.reset(),
                        }
                    }
                    None => {
                        refresh = false;
                    }
                }
            }
            Err(_) => {
                tick += 1;
            }
        }
    }
}

#[derive(PartialEq)]
pub enum Screen {
    Home,
    Graph,
    Status,
}

impl Screen {
    pub fn next(self) -> Self {
        match self {
            Screen::Home => Screen::Graph,
            Screen::Graph => Screen::Status,
            Screen::Status => Screen::Home,   // wrap around
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Screen::Home => Screen::Status,
            Screen::Graph => Screen::Home,
            Screen::Status => Screen::Graph,
        }
    }
}

use lcd_async::{raw_framebuf::RawFrameBuf, Display};
use embedded_graphics::{
    pixelcolor::{Rgb565, RgbColor},
    draw_target::DrawTarget,
};

pub async fn clear_screen<DI, MODEL, RST>(
    display: &mut Display<DI, MODEL, RST>,
    chunk_buffer: &mut [u8],
) 
where
        DI:    lcd_async::interface::Interface<Word = u8>,
        MODEL: lcd_async::models::Model<ColorFormat = Rgb565>,
        RST:   embedded_hal::digital::OutputPin,
{
    let mut y = 0usize;
    while y < DISPLAY_HEIGHT {
        let h = INIT_BAND_HEIGHT.min(DISPLAY_HEIGHT - y);
        
        let mut fb = RawFrameBuf::<Rgb565, _>::new(
            &mut *chunk_buffer,
            DISPLAY_WIDTH, h,
        );
        
        let _ = fb.clear(Rgb565::BLACK);

        let _ = display.show_raw_data(
            0,
            y as u16,
            DISPLAY_WIDTH as u16, h as u16,
            chunk_buffer
        ).await;

        y += h;
    }
}
