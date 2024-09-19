#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal::entry]
fn main() -> ! {
    esp_hal::rom_usb_print(b"!\n");
    core::panic!("haho panic\n");

    //loop {}
}
