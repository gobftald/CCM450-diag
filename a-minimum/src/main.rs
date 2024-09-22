#![no_std]
#![no_main]

use esp_println::dbg;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal::entry]
fn main() -> ! {
    let x = 2024;
    esp_println::println!("haho espressif {}", x);
    dbg!(x);

    loop {}
}
