#[cfg(all(
    not(feature = "defmt"),
    not(feature = "esp-backtrace"),
    not(feature = "no-op")
))]
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

#[cfg(any(
    feature = "no-op",
    all(feature = "defmt", not(feature = "esp-backtrace"))
))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

#[cfg(feature = "esp-backtrace")]
use esp_backtrace as _;
