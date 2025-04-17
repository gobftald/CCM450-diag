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
    panic!("panic in main of {=str} at {=u32}", file!(), line!())
}

// cargo run --release (console off, dfmt off)
// you need to comment esp_hal::print out (.text size is 0xce)

// cargo run --release (console on, dfmt off)
// esp_hal::print can be used and this is the only call to produce output

// cargo run --release (defmt on)
// esp_hal::print
// panic! (with its message)

// DEFMT_LOG=off cargo run --release (defmt on)
// esp_hal::print only
// panic! (without its message)

// DEFMT_LOG=debug cargo run --release (defmt on)
// esp_hal::print
// debug!
// panic! (with its message)

// all of the case above naturally with build-std-features = ["panic_immediate_abort"] (and panic = 'abort')
// this eliminate all core::fmt code and 'static text (e.g. the file and line info of a panic) or
// in case of defmt put them on the host side
// cargo run --release
// in .cargo/config.toml
// #"-C", "force-frame-pointers",
// build-std-features = ["panic_immediate_abort"]

// cargo run --release --features=backtrace
// in .cargo/config.toml
// "-C", "force-frame-pointers",
// #build-std-features = ["panic_immediate_abort"]

// this way we can configure 4 cases:
// 1 debug mode -> debug info)
// 2 release mode with defmt -> DEFMT_LOG filters manage 'debug! and panic! macros
// 3 release mode wo defmt -> defmt runtime and 'defmt::debug!' macros totally removed form code
// 4 release mode wo defmt wo console -> no output, no usb print and 'defmt::debug' code overhead
// --> minimal code size
// since in the 4 cases above we use build-std-features = ["panic_immediate_abort"], there is no core::fmt
// code and all 'internal/language' panics will goes to a minimal unimp/illegal intstruction
