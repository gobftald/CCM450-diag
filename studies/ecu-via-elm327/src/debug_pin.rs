use core::cell::OnceCell;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};

static mut DEBUG_PIN: OnceCell<Output> = OnceCell::new();

#[allow(dead_code)]
pub fn init_debug_pin(pin: impl OutputPin + 'static) {
    unsafe {
        DEBUG_PIN.get_mut_or_init(|| Output::new(pin, Level::High, OutputConfig::default()));
    }
}
#[allow(unused)]
pub fn debug_pin(level: u8) {
    unsafe {
        DEBUG_PIN
            .get_mut()
            .map(|pin| pin.set_level(if level == 0 { Level::Low } else { Level::High }))
    };
}
