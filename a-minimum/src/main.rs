#![no_std]
#![no_main]

// if we don't use esp_println directly
// we need to get inti scope for esp_hal macros
#[cfg(feature = "defmt")]
use esp_println as _;

#[cfg(not(feature = "defmt"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(location) = info.location() {
        esp_println::print!("panic at {}:{}: ", location.file(), location.line(),);
    } else {
        esp_println::print!("panic: ");
    }
    esp_println::println!("{}", info.message());

    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

#[cfg(feature = "defmt")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

#[esp_hal::entry]
fn main() -> ! {
    esp_hal::unwrap!(None::<u32>);
    loop {}
}
