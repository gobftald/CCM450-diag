use lcd_async::raw_framebuf::RawFrameBuf;
use embedded_graphics::{
    pixelcolor::Rgb565,
    draw_target::DrawTarget,
    prelude::RgbColor,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Home,
    Graph,
    Logs,
}

impl Screen {
    pub fn next(self) -> Self {
        match self {
            Screen::Home => Screen::Graph,
            Screen::Graph => Screen::Logs,
            Screen::Logs => Screen::Home,   // wrap around
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Screen::Home => Screen::Logs,
            Screen::Graph => Screen::Home,
            Screen::Logs => Screen::Graph,
        }
    }
}

pub fn draw_screen(
    fb: &mut RawFrameBuf<Rgb565, &mut [u8]>,
    screen: Screen,
    incr: i32
) {
    fb.clear(Rgb565::BLACK).unwrap();
    match screen {
        Screen::Home => super::home::draw_home(fb),
        Screen::Graph => super::graph::draw_graph(fb, incr),
        Screen::Logs => super::logs::draw_logs(fb),
    }
}

/*
// Tap Zones for In-Screen Buttons
fn handle_tap(screen: Screen, x: u16, y: u16, dirty: &mut bool) {
    match screen {
        Screen::Settings => {
            // Example: button drawn at x=80..160, y=200..240
            if (80..160).contains(&x) && (200..240).contains(&y) {
                // toggle a setting
                *dirty = true;
            }
        }
        _ => {}
    }
}
*/