#![no_std]
#![no_main]

mod panic_handlers;

#[esp_hal::entry]
fn main() -> ! {
    /*
    // console print to usb jtagserill (either core or defmt)
    esp_println::println!("Haho Espressif");
    // defmt log macros
    esp_hal::error!("defmt log test");
    loop {}
    */

    /*
    // if only this section is uncommented, we don't use esp_prinln before
    // bit acquire/release of defmt is defined there, so we need to bring in into scope
    #[cfg(feature = "defmt")]
    use esp_println as _;
    // panic and backtrace testing
    esp_hal::unwrap!(None::<u32>);
    loop {}
    */

    /*
    // exception backtrace testing
    #[no_mangle]
    #[link_section = ".text"]
    static UNIMP_CODE: [u8; 2] = *b"\x00\x00";
    let illegal_instruction: extern "C" fn() -> ! =
        unsafe { core::mem::transmute(&UNIMP_CODE as *const _ as *const ()) };
    illegal_instruction();
    */
}
