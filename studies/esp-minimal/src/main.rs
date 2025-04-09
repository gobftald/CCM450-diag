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

#[esp_hal::main]
fn main() -> ! {
    //loop {} // jmp (2 bytes)
    panic!(); // unimp (2 bytes)
}

// This is the most minimum (size of .text is 0xd8 bytes) runable esp application
// detailed analyses are in 'root'/docs/
