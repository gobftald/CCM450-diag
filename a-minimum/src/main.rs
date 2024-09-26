#![no_std]
#![no_main]

mod panic_handlers;

#[esp_hal::entry]
fn main() -> ! {
    // panic backtrace testing
    /*
    esp_hal::unwrap!(None::<u32>);
    loop {}
    */

    // exception backtrace testing
    #[no_mangle]
    #[link_section = ".text"]
    static UNIMP_CODE: [u8; 2] = *b"\x00\x00";
    let illegal_instruction: extern "C" fn() -> ! =
        unsafe { core::mem::transmute(&UNIMP_CODE as *const _ as *const ()) };
    illegal_instruction();
}
