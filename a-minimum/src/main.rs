#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal::entry]
fn main() -> ! {
    // if we will use it for real printing
    //esp_hal::rom_usb_print(b"!");

    esp_hal::rom_usb_print(&b'!');
    loop {}
}
