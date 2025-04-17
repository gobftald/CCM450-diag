#![no_std]
#![no_main]

#[macro_use(panic, debug)]
extern crate esp_hal;

#[cfg(feature = "backtrace")]
use esp_backtrace as _;

// since we config 'build-std-features = ["panic_immediate_abort"]'
// in no backtrace mode, this handler will be never called, but the
//compiler insists on it
#[cfg(not(feature = "backtrace"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

#[esp_hal::main]
fn main() -> ! {
    esp_hal::print(b"01234567890123456789012345678901234567890123456789012345678\n");
    debug!("debug {=u32}", 0x66);
    unsafe {
        core::arch::asm!("unimp");
    }
    panic!("panic in main")
}

// cargo run --release --features=backtrace (switch defmt on/off)
// in .cargo/config.toml
// "-C", "force-frame-pointers",
// #build-std-features = ["panic_immediate_abort"]
