#![no_std]
#![no_main]

// panic_handler
mod panic;

#[allow(unused_imports)]
#[macro_use(core_println, println, debug, panic)] // core_println for panic_handler in mod panic
extern crate console;

#[esp_hal::main]
fn main() -> ! {
    console::print(b"01234567890123456789012345678901234567890123456789012345678\n");

    println!("println from main 0x{:x}", 0x55);
    debug!("debug {=u32} 0x{:x}", 0x66, 0x77);

    // for testing exception
    unsafe {
        core::arch::asm!("unimp");
    }

    panic!("panic called from main")
}

// #build-std-features = ["panic_immediate_abort"], defmt off, console on/off
// cargo run --release
// cargo run --release --features=backtrace     # beacktrace switch console on by default
//                                              # it needs the console in all cases

// both 'build-std-features = ["panic_immediate_abort"]' and defmt are on
// cargo run --release
// cargo run --release --features=backtrace     # "panic_immediate_abort" off, "force-frame-pointers" on
//                                              # defmt on/off
