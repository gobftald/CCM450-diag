#![no_std]
#![no_main]

// since we config 'build-std-features = ["panic_immediate_abort"]'
// this handler will be never called, but the compiler insists on it
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("wfi") }
    }
}

fn main() {
    //println!("Hello, world!");
}
