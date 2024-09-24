#![no_std]
#![no_main]

#[panic_handler]
fn core_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal::entry]
fn main() -> ! {
    let x = 2024;

    esp_println::println!("haho esp");

    esp_println::error!("error log: {}", x);
    esp_println::warn!("warn log: {}", x);
    esp_println::info!("info log: {}", x);
    esp_println::debug!("debug log: {}", x);
    esp_println::trace!("trace log: {}", x);

    loop {}
}
