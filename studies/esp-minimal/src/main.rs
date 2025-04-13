#![no_std]
#![no_main]

mod dfmt;

// since we config 'build-std-features = ["panic_immediate_abort"]'
// this handler will be never called, but the compiler insists on it
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

#[esp_hal::main]
fn main() -> ! {
    dfmt::usb_print(b"01234567890123456789012345678901234567890123456789012345678\n");
    defmt::println!("haho {=bool} {=str}\n", true, "string");
    defmt::debug!("debug {=u32}", 0x66);

    //loop {} // jmp (2 bytes)
    defmt::panic!("panic in main of {=str} at {=u32}", file!(), line!())
}

// DEFMT_LOG=debug cargo run --release
