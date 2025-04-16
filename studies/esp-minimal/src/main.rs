#![no_std]
#![no_main]

mod console;

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
    console::print(b"01234567890123456789012345678901234567890123456789012345678\n");
    defmt::println!("haho {=bool} {=str}\n", true, "string");
    defmt::debug!("debug {=u32}", 0x66);

    //loop {} // jmp (2 bytes)
    // DEFMT_LOG=off -> this panic text is not shown
    defmt::panic!("panic in main of {=str} at {=u32}", file!(), line!())
}

// cargo run --release --features=backtrace
// in .cargo/config.toml
// "-C", "force-frame-pointers",
// #build-std-features = ["panic_immediate_abort"]

// cargo run --release
// in .cargo/config.toml
// #"-C", "force-frame-pointers",
// build-std-features = ["panic_immediate_abort"]
