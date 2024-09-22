#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal::entry]
fn main() -> ! {
    let x = 2024;

    esp_println::logger::init_logger_from_env();

    log::error!("debug log: {}", x);
    log::warn!("debug log: {}", x);
    log::info!("debug log: {}", x);
    log::debug!("debug log: {}", x);
    log::trace!("debug log: {}", x);

    loop {}
}
